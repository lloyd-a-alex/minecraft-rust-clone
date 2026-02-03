use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;
use std::time::Instant;
use std::collections::HashMap;

use crate::world::{World, BlockPos, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use crate::player::{Player, HOTBAR_SIZE};
use crate::texture::TextureAtlas;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub pad: u32,
    pub tex_coords: [f32; 2],
    pub ao: f32,
    pub tex_index: u32,
}

impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute { offset: 0,  shader_location: 0, format: VertexFormat::Float32x3 },
                VertexAttribute { offset: 16, shader_location: 1, format: VertexFormat::Float32x2 },
                VertexAttribute { offset: 24, shader_location: 2, format: VertexFormat::Float32 },
                VertexAttribute { offset: 28, shader_location: 3, format: VertexFormat::Uint32 },
            ],
        }
    }
}

pub struct ChunkMesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    depth_texture: Texture,
    depth_view: TextureView,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    time_buffer: Buffer,
    time_bind_group: BindGroup,
    texture_bind_group: BindGroup,
    start_time: Instant,
    pub chunk_meshes: HashMap<(i32, i32), ChunkMesh>,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::empty(),
            dx12_shader_compiler: Dx12Compiler::Fxc,
            gles_minor_version: Gles3MinorVersion::Automatic,
        });
        
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::downlevel_defaults(),
        }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo, 
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // --- TEXTURE ATLAS ---
        let atlas = TextureAtlas::new();
        let texture_size = Extent3d { width: atlas.size, height: atlas.size, depth_or_array_layers: 1 };
        let texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("atlas_texture"), view_formats: &[],
        });
        queue.write_texture(
            ImageCopyTexture { texture: &texture, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All },
            &atlas.data,
            ImageDataLayout { offset: 0, bytes_per_row: Some(4 * atlas.size), rows_per_image: Some(atlas.size) },
            texture_size,
        );
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Nearest, min_filter: FilterMode::Nearest, mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge, address_mode_v: AddressMode::ClampToEdge,
            ..Default::default()
        });

        // --- BUFFERS ---
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"), contents: bytemuck::cast_slice(&[[0.0f32; 16]]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"), contents: bytemuck::cast_slice(&[0.0f32; 8]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // --- BIND GROUPS ---
        let texture_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::FRAGMENT, ty: BindingType::Texture { multisampled: false, view_dimension: TextureViewDimension::D2, sample_type: TextureSampleType::Float { filterable: true } }, count: None },
                BindGroupLayoutEntry { binding: 1, visibility: ShaderStages::FRAGMENT, ty: BindingType::Sampler(SamplerBindingType::Filtering), count: None },
            ], label: Some("tex_bg_layout"),
        });
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bg_layout,
            entries: &[BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) }, BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&sampler) }],
            label: Some("tex_bg"),
        });

        let camera_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::VERTEX, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }], label: Some("cam_bg_layout"),
        });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }], label: Some("cam_bg"),
        });

        let time_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }], label: Some("time_bg_layout"),
        });
        let time_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &time_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: time_buffer.as_entire_binding() }], label: Some("time_bg"),
        });

        // --- PIPELINE ---
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"), bind_group_layouts: &[&texture_bg_layout, &camera_bg_layout, &time_bg_layout], push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc()] },
            fragment: Some(FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState { format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL })] }),
            primitive: PrimitiveState { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), unclipped_depth: false, polygon_mode: PolygonMode::Fill, conservative: false },
            depth_stencil: Some(DepthStencilState { format: TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: CompareFunction::Less, stencil: StencilState::default(), bias: DepthBiasState::default() }),
            multisample: MultisampleState::default(), multiview: None,
        });

        let depth_texture = device.create_texture(&TextureDescriptor {
            size: Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        Self {
            surface, device, queue, config, size, render_pipeline, depth_texture, depth_view,
            camera_buffer, camera_bind_group, time_buffer, time_bind_group, texture_bind_group,
            start_time: Instant::now(), chunk_meshes: HashMap::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.depth_texture = self.device.create_texture(&TextureDescriptor {
                size: Extent3d { width: self.config.width, height: self.config.height, depth_or_array_layers: 1 },
                mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[],
            });
            self.depth_view = self.depth_texture.create_view(&TextureViewDescriptor::default());
        }
    }

    pub fn update_camera(&mut self, view_proj: [[f32; 4]; 4]) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
    }

    pub fn update_chunk(&mut self, cx: i32, cz: i32, world: &World) {
        if let Some(chunk) = world.chunks.get(&(cx, cz)) {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let mut idx_offset = 0;

            for x in 0..CHUNK_SIZE_X {
                for y in 0..CHUNK_SIZE_Y {
                    for z in 0..CHUNK_SIZE_Z {
                        let block = chunk.get_block(x, y, z);
                        if !block.is_solid() && !block.is_water() { continue; }
                        
                        let wx = (cx * CHUNK_SIZE_X as i32 + x as i32) as f32;
                        let wy = y as f32;
                        let wz = (cz * CHUNK_SIZE_Z as i32 + z as i32) as f32;
                        let (top, side, bot) = block.get_texture_indices();
                        let w_pos = BlockPos { x: wx as i32, y: wy as i32, z: wz as i32 };

                        let check_face = |nx, ny, nz| {
                            let neighbor = world.get_block(BlockPos { x: nx, y: ny, z: nz });
                            if block.is_water() {
                                // Draw water faces against air OR when looking out from inside water to air
                                !neighbor.is_water() && !neighbor.is_solid()
                            } else {
                                // Draw solid faces against nonsolid (including water)
                                !neighbor.is_solid()
                            }
                        };

                        if check_face(w_pos.x, w_pos.y+1, w_pos.z) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx, wy+1.0, wz], [wx+1.0, wy+1.0, wz], [wx+1.0, wy+1.0, wz+1.0], [wx, wy+1.0, wz+1.0], top); }
                        if check_face(w_pos.x, w_pos.y-1, w_pos.z) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx, wy, wz+1.0], [wx+1.0, wy, wz+1.0], [wx+1.0, wy, wz], [wx, wy, wz], bot); }
                        if check_face(w_pos.x, w_pos.y, w_pos.z+1) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx, wy+1.0, wz+1.0], [wx+1.0, wy+1.0, wz+1.0], [wx+1.0, wy, wz+1.0], [wx, wy, wz+1.0], side); }
                        if check_face(w_pos.x, w_pos.y, w_pos.z-1) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx+1.0, wy+1.0, wz], [wx, wy+1.0, wz], [wx, wy, wz], [wx+1.0, wy, wz], side); }
                        if check_face(w_pos.x+1, w_pos.y, w_pos.z) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx+1.0, wy+1.0, wz+1.0], [wx+1.0, wy+1.0, wz], [wx+1.0, wy, wz], [wx+1.0, wy, wz+1.0], side); }
                        if check_face(w_pos.x-1, w_pos.y, w_pos.z) { self.add_quad(&mut vertices, &mut indices, &mut idx_offset, [wx, wy+1.0, wz], [wx, wy+1.0, wz+1.0], [wx, wy, wz+1.0], [wx, wy, wz], side); }
                    }
                }
            }

            if !vertices.is_empty() {
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk VB"), contents: bytemuck::cast_slice(&vertices), usage: BufferUsages::VERTEX });
                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk IB"), contents: bytemuck::cast_slice(&indices), usage: BufferUsages::INDEX });
                self.chunk_meshes.insert((cx, cz), ChunkMesh { vertex_buffer, index_buffer, index_count: indices.len() as u32 });
            }
        }
    }

    fn add_cube(&self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, offset: &mut u32, x: f32, y: f32, z: f32, top: u32, side: u32, bot: u32, scale: f32) {
        let s = scale;
        let pts = [
            [x, y+s, z], [x+s, y+s, z], [x+s, y, z], [x, y, z], // Back
            [x, y+s, z+s], [x+s, y+s, z+s], [x+s, y, z+s], [x, y, z+s], // Front
        ];
        self.add_quad(vertices, indices, offset, pts[4], pts[5], pts[6], pts[7], side);
        self.add_quad(vertices, indices, offset, pts[1], pts[0], pts[3], pts[2], side);
        self.add_quad(vertices, indices, offset, pts[0], pts[1], pts[5], pts[4], top);
        self.add_quad(vertices, indices, offset, pts[7], pts[6], pts[2], pts[3], bot);
        self.add_quad(vertices, indices, offset, pts[5], pts[1], pts[2], pts[6], side);
        self.add_quad(vertices, indices, offset, pts[0], pts[4], pts[7], pts[3], side);
    }

    fn add_quad(&self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, offset: &mut u32, p0: [f32;3], p1: [f32;3], p2: [f32;3], p3: [f32;3], tex: u32) {
        vertices.push(Vertex { position: p0, pad: 0, tex_coords: [0.0, 0.0], ao: 1.0, tex_index: tex });
        vertices.push(Vertex { position: p1, pad: 0, tex_coords: [1.0, 0.0], ao: 1.0, tex_index: tex });
        vertices.push(Vertex { position: p2, pad: 0, tex_coords: [1.0, 1.0], ao: 1.0, tex_index: tex });
        vertices.push(Vertex { position: p3, pad: 0, tex_coords: [0.0, 1.0], ao: 1.0, tex_index: tex });
        indices.push(*offset); indices.push(*offset + 3); indices.push(*offset + 2);
        indices.push(*offset); indices.push(*offset + 2); indices.push(*offset + 1);
        *offset += 4;
    }

    pub fn render(&mut self, player: &Player, world: &World, is_underwater: bool) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Outdated) => { self.resize(self.size); return; }
            Err(e) => { eprintln!("Render error: {:?}", e); return; }
        };
        
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render Encoder") });

        let time = self.start_time.elapsed().as_secs_f32();
        let underwater_float = if is_underwater { 1.0 } else { 0.0 };
        // Update Time Buffer: [color_r, color_g, color_b, alpha, time, underwater_flag, pad, pad]
        self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[0.5, 0.8, 0.9, 1.0, time, underwater_float, 0.0, 0.0]));

        // --- DYNAMIC GEOMETRY (Entities) ---
        let mut dyn_verts = Vec::new();
        let mut dyn_inds = Vec::new();
        let mut dyn_off = 0;

        for e in &world.entities {
             let (t, _, _) = e.item_type.get_texture_indices();
             let bob = ((time * 4.0 + e.bob_offset).sin() * 0.1) + 0.1;
             self.add_cube(&mut dyn_verts, &mut dyn_inds, &mut dyn_off, e.position.x - 0.125, e.position.y + bob, e.position.z - 0.125, t, t, t, 0.25);
        }
        
        // --- HUD / UI GEOMETRY ---
        // Moved closer to 0.4 (camera near is 0.1) to avoid clipping and depth fighting.
        let (sin, cos) = player.rotation.x.sin_cos();
        let (y_sin, y_cos) = player.rotation.y.sin_cos();
        
        let fwd = [y_cos * cos, sin, y_sin * cos];
        let right = [-y_sin, 0.0, y_cos];
        let up = [y_cos * -sin, cos, y_sin * -sin]; 

        let cam_x = player.position.x;
        let cam_y = player.position.y + player.height * 0.9;
        let cam_z = player.position.z;
        let dist = 0.4; // Fixed visibility issue (was 0.15, clipping near plane)

        let mut draw_hud_element = |offset_x: f32, offset_y: f32, scale: f32, tex: u32| {
             let cx = cam_x + fwd[0]*dist + right[0]*offset_x + up[0]*offset_y;
             let cy = cam_y + fwd[1]*dist + right[1]*offset_x + up[1]*offset_y;
             let cz = cam_z + fwd[2]*dist + right[2]*offset_x + up[2]*offset_y;
             self.add_cube(&mut dyn_verts, &mut dyn_inds, &mut dyn_off, cx, cy, cz, tex, tex, tex, scale);
        };

        if player.inventory_open {
             // Menu Background
             draw_hud_element(0.0, 0.0, 0.3, 253);
        } else {
            // 1. Crosshair
            draw_hud_element(0.0, 0.0, 0.005, 254);
            
            // 2. Hotbar
            let hb_y = -0.15;
            let spacing = 0.03;
            let start_x = -(HOTBAR_SIZE as f32 / 2.0) * spacing + (spacing / 2.0);
            
            for i in 0..HOTBAR_SIZE {
                let x = start_x + (i as f32 * spacing);
                let tex = if i == player.inventory.selected_hotbar_slot { 251 } else { 250 };
                draw_hud_element(x, hb_y, 0.025, tex);
                
                if let Some(stack) = player.inventory.slots[i] {
                    let (t, _, _) = stack.item.get_texture_indices();
                    draw_hud_element(x, hb_y, 0.015, t);
                }
            }
            
            // 3. Hearts
            let pulse = (time * 5.0).sin() * 0.002;
            let h_y = hb_y + 0.05;
            let h_start_x = -0.12;
            for i in 0..10 {
                if (i as f32) < (player.health / 2.0) {
                     draw_hud_element(h_start_x + (i as f32 * 0.02), h_y + pulse, 0.008, 252);
                }
            }
        }

        let dyn_vb = if !dyn_verts.is_empty() { Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Dyn VB"), contents: bytemuck::cast_slice(&dyn_verts), usage: BufferUsages::VERTEX })) } else { None };
        let dyn_ib = if !dyn_inds.is_empty() { Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Dyn IB"), contents: bytemuck::cast_slice(&dyn_inds), usage: BufferUsages::INDEX })) } else { None };

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: Operations { load: LoadOp::Clear(Color { r: 0.5, g: 0.8, b: 0.9, a: 1.0 }), store: StoreOp::Store },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.texture_bind_group, &[]);
            rpass.set_bind_group(1, &self.camera_bind_group, &[]);
            rpass.set_bind_group(2, &self.time_bind_group, &[]);

            for mesh in self.chunk_meshes.values() {
                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                rpass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
            
            if let (Some(vb), Some(ib)) = (&dyn_vb, &dyn_ib) {
                rpass.set_vertex_buffer(0, vb.slice(..));
                rpass.set_index_buffer(ib.slice(..), IndexFormat::Uint32);
                rpass.draw_indexed(0..dyn_inds.len() as u32, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}