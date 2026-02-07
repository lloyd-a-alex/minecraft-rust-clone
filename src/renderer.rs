use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;
use std::time::Instant;
use std::collections::HashMap;
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, BlockPos, BlockType};
use crate::player::Player;
use crate::texture::TextureAtlas;
use crate::MainMenu; // Added Rect

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle { pub pos: glam::Vec3, pub vel: glam::Vec3, pub life: f32, pub color_idx: u32 }

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex { pub position: [f32; 3], pub tex_coords: [f32; 2], pub ao: f32, pub tex_index: u32, pub light: f32 }
impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress, step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute { offset: 0, shader_location: 0, format: VertexFormat::Float32x3 }, 
                VertexAttribute { offset: 12, shader_location: 1, format: VertexFormat::Float32x2 }, 
                VertexAttribute { offset: 20, shader_location: 2, format: VertexFormat::Float32 }, 
                VertexAttribute { offset: 24, shader_location: 3, format: VertexFormat::Uint32 },
                VertexAttribute { offset: 28, shader_location: 4, format: VertexFormat::Float32 }
            ],
        }
    }
}

pub struct ChunkMesh { vertex_buffer: Buffer, index_buffer: Buffer, index_count: u32 }

pub struct Renderer<'a> {
    pub particles: Vec<Particle>,
surface: Surface<'a>, device: Device, queue: Queue, pub config: SurfaceConfiguration,
    pipeline: RenderPipeline, ui_pipeline: RenderPipeline,
    depth_texture: TextureView, bind_group: BindGroup,
    camera_buffer: Buffer, camera_bind_group: BindGroup,
    time_buffer: Buffer, time_bind_group: BindGroup,
    pub start_time: Instant, chunk_meshes: HashMap<(i32, i32), ChunkMesh>,
    entity_vertex_buffer: Buffer, entity_index_buffer: Buffer,
    pub break_progress: f32,
}

impl<'a> Renderer<'a> {
    pub fn update_camera(&mut self, player: &Player, aspect: f32) {
    let view_proj = player.build_view_projection_matrix(aspect);
    self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
}
    pub async fn new(window: &'a Window) -> Self {
        // --- KEY FIX: Use empty flags for compatibility ---
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&RequestAdapterOptions { power_preference: PowerPreference::HighPerformance, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default(), None).await.unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration { usage: TextureUsages::RENDER_ATTACHMENT, format: surface_format, width: window.inner_size().width, height: window.inner_size().height, present_mode: PresentMode::Fifo, alpha_mode: surface_caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &config);

        let atlas = TextureAtlas::new();
        let atlas_size = Extent3d { width: 512, height: 512, depth_or_array_layers: 1 };
        let texture = device.create_texture(&TextureDescriptor { label: Some("atlas"), size: atlas_size, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Rgba8UnormSrgb, usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, view_formats: &[] });
        queue.write_texture(ImageCopyTexture { texture: &texture, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All }, &atlas.data, ImageDataLayout { offset: 0, bytes_per_row: Some(512 * 4), rows_per_image: Some(512) }, atlas_size);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor { mag_filter: FilterMode::Nearest, min_filter: FilterMode::Nearest, mipmap_filter: FilterMode::Nearest, ..Default::default() });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: Some("texture_layout"), entries: &[BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::FRAGMENT, ty: BindingType::Texture { sample_type: TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false }, count: None }, BindGroupLayoutEntry { binding: 1, visibility: ShaderStages::FRAGMENT, ty: BindingType::Sampler(SamplerBindingType::Filtering), count: None }] });
        let bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("texture_bind"), layout: &bind_group_layout, entries: &[BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) }, BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&sampler) }] });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Camera Buffer"), contents: bytemuck::cast_slice(&[[0.0f32; 16]]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST });
        let camera_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: Some("camera_layout"), entries: &[BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::VERTEX, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }] });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("camera_bind"), layout: &camera_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }] });

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Time Buffer"), contents: bytemuck::cast_slice(&[0.0f32; 8]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST });
        let time_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: Some("time_layout"), entries: &[BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::FRAGMENT, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }] });
        let time_bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("time_bind"), layout: &time_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: time_buffer.as_entire_binding() }] });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("Pipeline Layout"), bind_group_layouts: &[&bind_group_layout, &camera_bg_layout, &time_bg_layout], push_constant_ranges: &[] });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor { label: Some("Pipeline"), layout: Some(&pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc()] }, fragment: Some(FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState { format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL })] }), primitive: PrimitiveState { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), ..Default::default() }, depth_stencil: Some(DepthStencilState { format: TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: CompareFunction::Less, stencil: StencilState::default(), bias: DepthBiasState::default() }), multisample: MultisampleState::default(), multiview: None });

        let ui_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("UI Layout"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[] });
        let ui_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor { label: Some("UI Pipeline"), layout: Some(&ui_pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_ui", buffers: &[Vertex::desc()] }, fragment: Some(FragmentState { module: &shader, entry_point: "fs_ui", targets: &[Some(ColorTargetState { format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL })] }), primitive: PrimitiveState { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: None, ..Default::default() }, depth_stencil: None, multisample: MultisampleState::default(), multiview: None });

        let depth_texture = device.create_texture(&TextureDescriptor { size: Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float, usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[] }).create_view(&TextureViewDescriptor::default());
        let entity_vertex_buffer = device.create_buffer(&BufferDescriptor { label: Some("Entity VB"), size: 1024, usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, mapped_at_creation: false });
        let entity_index_buffer = device.create_buffer(&BufferDescriptor { label: Some("Entity IB"), size: 1024, usage: BufferUsages::INDEX | BufferUsages::COPY_DST, mapped_at_creation: false });

        Self { particles: Vec::new(), surface, device, queue, config, pipeline, ui_pipeline, depth_texture, bind_group, camera_bind_group, camera_buffer, time_bind_group, time_buffer, start_time: Instant::now(), chunk_meshes: HashMap::new(), entity_vertex_buffer, entity_index_buffer, break_progress: 0.0 }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width; self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = self.device.create_texture(&TextureDescriptor { size: Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float, usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[] }).create_view(&TextureViewDescriptor::default());
        }
    }

pub fn rebuild_all_chunks(&mut self, world: &World) { 
        self.chunk_meshes.clear(); 
        for (key, _) in &world.chunks { self.update_chunk(key.0, key.1, world); }
        self.update_clouds(world);
    }

    fn update_clouds(&mut self, world: &World) {
        let mut vertices = Vec::new(); let mut indices = Vec::new(); let mut offset = 0;
        let cloud_y = 110.0;
        for cx in -10..10 {
            for cz in -10..10 {
                let wx = cx * 16; let wz = cz * 16;
                let noise = crate::noise_gen::NoiseGenerator::new(world.seed);
                if noise.get_noise3d(wx as f64 * 0.01, 0.0, wz as f64 * 0.01) > 0.4 {
                    self.add_face(&mut vertices, &mut indices, &mut offset, wx, cloud_y as i32, wz, 0, 228, 1.0, 1.0);
                    self.add_face(&mut vertices, &mut indices, &mut offset, wx, cloud_y as i32, wz, 1, 228, 1.0, 0.8);
                }
            }
        }
        if !vertices.is_empty() {
            let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Cloud VB"), contents: bytemuck::cast_slice(&vertices), usage: BufferUsages::VERTEX });
            let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Cloud IB"), contents: bytemuck::cast_slice(&indices), usage: BufferUsages::INDEX });
            self.chunk_meshes.insert((999, 999), ChunkMesh { vertex_buffer: vb, index_buffer: ib, index_count: indices.len() as u32 });
        }
    }

pub fn update_chunk(&mut self, cx: i32, cz: i32, world: &World) {
        self.chunk_meshes.remove(&(cx, cz));
        if let Some(chunk) = world.chunks.get(&(cx, cz)) {
            let mut vertices = Vec::new(); 
            let mut indices = Vec::new(); 
            let mut index_offset = 0;
            let chunk_x = cx * CHUNK_SIZE_X as i32; 
            let chunk_z = cz * CHUNK_SIZE_Z as i32;
            for x in 0..CHUNK_SIZE_X { 
                for y in 0..CHUNK_SIZE_Y { 
                    for z in 0..CHUNK_SIZE_Z {
                        let block = chunk.get_block(x, y, z);
                        if block == BlockType::Air { continue; }
                        let (tex_top, tex_bot, tex_side) = block.get_texture_indices();
                        let wx = chunk_x + x as i32; 
                        let wy = y as i32; 
                        let wz = chunk_z + z as i32;
                        let h = if block.is_water() { if block.get_water_level() == 8 { 1.0 } else { block.get_water_level() as f32 / 9.0 + 0.1 } } else { 1.0 };
                        let light = world.get_light_world(BlockPos { x: wx, y: wy, z: wz }) as f32 / 15.0;
                        
                        let check = |dx, dy, dz| { 
                            let n = world.get_block(BlockPos { x: wx+dx, y: wy+dy, z: wz+dz }); 
                            n == BlockType::Air || (n.is_transparent() && n != block) 
                        };
                        
                        if check(0, 1, 0) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 0, tex_top, h, light); }
                        if check(0, -1, 0) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 1, tex_bot, h, light); }
                        if check(1, 0, 0) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 3, tex_side, h, light); }
                        if check(-1, 0, 0) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 2, tex_side, h, light); }
                        if check(0, 0, 1) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 4, tex_side, h, light); }
                        if check(0, 0, -1) { self.add_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, 5, tex_side, h, light); }
                        
                        if block.is_cross_model() {
                             let (t, _, _) = block.get_texture_indices();
                             self.add_cross_face(&mut vertices, &mut indices, &mut index_offset, wx, wy, wz, t, light);
                        }
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

fn add_face(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32, x: i32, y: i32, z: i32, face: u8, tex: u32, h: f32, light: f32) {
        let x = x as f32; let y = y as f32; let z = z as f32;
        let (p0, p1, p2, p3, uv0, uv1, uv2, uv3) = match face {
            0 => ([x,y+h,z+1.0], [x+1.0,y+h,z+1.0], [x+1.0,y+h,z], [x,y+h,z], [0.0,1.0], [1.0,1.0], [1.0,0.0], [0.0,0.0]),
            1 => ([x,y,z], [x+1.0,y,z], [x+1.0,y,z+1.0], [x,y,z+1.0], [0.0,0.0], [1.0,0.0], [1.0,1.0], [0.0,1.0]),
            2 => ([x,y,z], [x,y,z+1.0], [x,y+h,z+1.0], [x,y+h,z], [0.0,1.0], [1.0,1.0], [1.0,0.0], [0.0,0.0]),
            3 => ([x+1.0,y,z+1.0], [x+1.0,y,z], [x+1.0,y+h,z], [x+1.0,y+h,z+1.0], [0.0,1.0], [1.0,1.0], [1.0,0.0], [0.0,0.0]),
            4 => ([x,y,z+1.0], [x+1.0,y,z+1.0], [x+1.0,y+h,z+1.0], [x,y+h,z+1.0], [0.0,1.0], [1.0,1.0], [1.0,0.0], [0.0,0.0]),
            5 => ([x+1.0,y,z], [x,y,z], [x,y+h,z], [x+1.0,y+h,z], [0.0,1.0], [1.0,1.0], [1.0,0.0], [0.0,0.0]),
            _ => return,
        };
        v.push(Vertex{position:p0, tex_coords:uv0, ao:1.0, tex_index:tex, light}); 
        v.push(Vertex{position:p1, tex_coords:uv1, ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:p2, tex_coords:uv2, ao:1.0, tex_index:tex, light}); 
        v.push(Vertex{position:p3, tex_coords:uv3, ao:1.0, tex_index:tex, light});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
    }

    fn add_rotated_quad(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32, c: [f32;3], rot: f32, rx: f32, ry: f32, rz: f32, s: f32, face: usize, tex: u32) {
        let (sin, cos) = rot.sin_cos();
        let rotate = |x: f32, z: f32| (x * cos - z * sin, x * sin + z * cos);
        let (p0, p1, p2, p3) = match face {
            0 => ((rx,ry+s,rz+s), (rx+s,ry+s,rz+s), (rx+s,ry+s,rz), (rx,ry+s,rz)),
            1 => ((rx,ry,rz), (rx+s,ry,rz), (rx+s,ry,rz+s), (rx,ry,rz+s)),
            2 => ((rx,ry,rz), (rx,ry,rz+s), (rx,ry+s,rz+s), (rx,ry+s,rz)),
            3 => ((rx+s,ry,rz+s), (rx+s,ry,rz), (rx+s,ry+s,rz), (rx+s,ry+s,rz+s)),
            4 => ((rx,ry,rz+s), (rx+s,ry,rz+s), (rx+s,ry+s,rz+s), (rx,ry+s,rz+s)),
            5 => ((rx+s,ry,rz), (rx,ry,rz), (rx,ry+s,rz), (rx+s,ry+s,rz)), _ => return,
        };
        let t = |p: (f32,f32,f32)| { let r = rotate(p.0, p.2); [c[0]+r.0, c[1]+p.1, c[2]+r.1] };
        v.push(Vertex{position:t(p0), tex_coords:[0.0,1.0], ao:1.0, tex_index:tex, light: 1.0}); v.push(Vertex{position:t(p1), tex_coords:[1.0,1.0], ao:1.0, tex_index:tex, light: 1.0});
        v.push(Vertex{position:t(p2), tex_coords:[1.0,0.0], ao:1.0, tex_index:tex, light: 1.0}); v.push(Vertex{position:t(p3), tex_coords:[0.0,0.0], ao:1.0, tex_index:tex, light: 1.0});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
    }

fn add_cross_face(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32, x: i32, y: i32, z: i32, tex: u32, light: f32) {
        let x = x as f32; let y = y as f32; let z = z as f32;
        // Diagonal 1
        v.push(Vertex{position:[x, y+1.0, z], tex_coords:[0.0,0.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x+1.0, y+1.0, z+1.0], tex_coords:[1.0,0.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x+1.0, y, z+1.0], tex_coords:[1.0,1.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x, y, z], tex_coords:[0.0,1.0], ao:1.0, tex_index:tex, light});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
        
        // Diagonal 2
        v.push(Vertex{position:[x, y+1.0, z+1.0], tex_coords:[0.0,0.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x+1.0, y+1.0, z], tex_coords:[1.0,0.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x+1.0, y, z], tex_coords:[1.0,1.0], ao:1.0, tex_index:tex, light});
        v.push(Vertex{position:[x, y, z+1.0], tex_coords:[0.0,1.0], ao:1.0, tex_index:tex, light});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
    }

fn add_ui_quad(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32, x: f32, y: f32, w: f32, h: f32, tex: u32) {
        v.push(Vertex{position:[x,y+h,0.0], tex_coords:[0.0,0.0], ao:1.0, tex_index:tex, light: 1.0}); v.push(Vertex{position:[x+w,y+h,0.0], tex_coords:[1.0,0.0], ao:1.0, tex_index:tex, light: 1.0});
        v.push(Vertex{position:[x+w,y,0.0], tex_coords:[1.0,1.0], ao:1.0, tex_index:tex, light: 1.0}); v.push(Vertex{position:[x,y,0.0], tex_coords:[0.0,1.0], ao:1.0, tex_index:tex, light: 1.0});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
    }

fn draw_text(&self, text: &str, start_x: f32, y: f32, scale: f32, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let mut x = start_x;
        let mut final_scale = scale;
        if text.len() > 10 { final_scale *= 0.8; }

        for c in text.to_uppercase().chars() {
            if c == ' ' { x += final_scale; continue; }
let idx = if c >= 'A' && c <= 'Z' { 300 + (c as u32 - 'A' as u32) } 
                      else if c >= '0' && c <= '9' { 300 + 26 + (c as u32 - '0' as u32) } 
                      else if c == '-' { 300 + 36 } 
                      else if c == '>' { 300 + 37 } 
                      else { 300 };
            
            self.add_ui_quad(v, i, off, x, y, final_scale, final_scale * aspect, idx);
            x += final_scale;
        }
    }
pub fn render_main_menu(&mut self, menu: &MainMenu, _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Menu") });

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut idx_offset = 0;

// 1. Background (Tile Dirt - classic darkened look)
    let grid_count = 10;
    let grid_size = 2.0 / grid_count as f32;
    for gy in 0..grid_count {
        for gx in 0..grid_count {
            let rx = -1.0 + (gx as f32 * grid_size) + grid_size/2.0;
            let ry = -1.0 + (gy as f32 * grid_size) + grid_size/2.0;
let tex_id = 1u32;
            let u_min = (tex_id % 32) as f32 / 32.0; let v_min = (tex_id / 32) as f32 / 32.0;
            let u_max = u_min + 1.0 / 32.0; let v_max = v_min + 1.0 / 32.0;
            let ao = 0.4;
            vertices.push(Vertex { position: [rx - grid_size / 2.0, ry - grid_size / 2.0, 0.0], tex_coords: [u_min, v_max], ao, tex_index: tex_id, light: 1.0 });
            vertices.push(Vertex { position: [rx + grid_size / 2.0, ry - grid_size / 2.0, 0.0], tex_coords: [u_max, v_max], ao, tex_index: tex_id, light: 1.0 });
            vertices.push(Vertex { position: [rx + grid_size / 2.0, ry + grid_size / 2.0, 0.0], tex_coords: [u_max, v_min], ao, tex_index: tex_id, light: 1.0 });
            vertices.push(Vertex { position: [rx - grid_size / 2.0, ry + grid_size / 2.0, 0.0], tex_coords: [u_min, v_min], ao, tex_index: tex_id, light: 1.0 });
            indices.extend_from_slice(&[idx_offset, idx_offset + 1, idx_offset + 2, idx_offset, idx_offset + 2, idx_offset + 3]);
            idx_offset += 4;
        }
    }

// 1.5 Draw Title (DIABOLICALLY BIG)
    self.draw_text("MINECRAFT", -0.85, 0.7, 0.2, &mut vertices, &mut indices, &mut idx_offset);

    // 2. Buttons & Text
    for btn in &menu.buttons {
        let tex_id = if btn.hovered { 251 } else { 250 };
        let u_min = (tex_id % 32) as f32 / 32.0; let v_min = (tex_id / 32) as f32 / 32.0;
        let u_max = u_min + 1.0 / 32.0; let v_max = v_min + 1.0 / 32.0;
        let rect = &btn.rect;
        
        // Button Quad
        vertices.push(Vertex { position: [rect.x - rect.w / 2.0, rect.y - rect.h / 2.0, 0.0], tex_coords: [u_min, v_max], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x + rect.w / 2.0, rect.y - rect.h / 2.0, 0.0], tex_coords: [u_max, v_max], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, 0.0], tex_coords: [u_max, v_min], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x - rect.w / 2.0, rect.y + rect.h / 2.0, 0.0], tex_coords: [u_min, v_min], ao: 1.0, tex_index: tex_id, light: 1.0 });
        indices.extend_from_slice(&[idx_offset, idx_offset + 1, idx_offset + 2, idx_offset, idx_offset + 2, idx_offset + 3]);
        idx_offset += 4;

        // Button Text (Centered and readable)
        let text_scale = 0.06;
        let center_offset = (btn.text.len() as f32 * text_scale) / 2.0;
        self.draw_text(&btn.text, rect.x - center_offset, rect.y - 0.02, text_scale, &mut vertices, &mut indices, &mut idx_offset);
    }

    // Render Pass
    let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Menu VB"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
    let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Menu IB"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Menu Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.ui_pipeline);
        // NOTE: Assuming your struct has 'diffuse_bind_group' based on typical naming. 
        // If error persists, change to 'bind_group' or 'texture_bind_group'.
rpass.set_bind_group(0, &self.bind_group, &[]);
        // REMOVED camera and time bind groups as UI pipeline doesn't use them in this version
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

pub fn render_pause_menu(&mut self, menu: &MainMenu, world: &World, player: &Player, cursor_pos: (f64, f64), width: u32, height: u32) -> Result<(), wgpu::SurfaceError> {
    // First render the world as a background
    self.render(world, player, true, cursor_pos, width, height)?;
    
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Pause") });

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut idx_offset = 0;

    // Dim overlay
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, -1.0, -1.0, 2.0, 2.0, 240);

    for btn in &menu.buttons {
        let tex_id = if btn.hovered { 251 } else { 250 };
        let rect = &btn.rect;
        self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, rect.x - rect.w/2.0, rect.y - rect.h/2.0, rect.w, rect.h, tex_id);
        
        let tx = (rect.x + 1.0) * 0.5 * width as f32;
        let ty = (1.0 - rect.y) * 0.5 * height as f32;
        let tw = btn.text.len() as f32 * 14.0;
        self.draw_text(&btn.text, (tx - tw / 2.0) / width as f32 * 2.0 - 1.0, 1.0 - (ty - 10.0) / height as f32 * 2.0, 0.003, &mut vertices, &mut indices, &mut idx_offset);
    }

    let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Pause VB"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
    let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Pause IB"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Pause Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.ui_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

pub fn render(&mut self, world: &World, player: &Player, is_paused: bool, cursor_pos: (f64, f64), _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let view_proj = player.build_view_projection_matrix(self.config.width as f32 / self.config.height as f32);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
let time = self.start_time.elapsed().as_secs_f32();
        let eye_bp = BlockPos { x: player.position.x.floor() as i32, y: (player.position.y + player.height * 0.4).floor() as i32, z: player.position.z.floor() as i32 };
let is_underwater = if world.get_block(eye_bp).is_water() { 1.0f32 } else { 0.0f32 };
        
// BIOME FOG LOGIC
        let noise_gen = crate::noise_gen::NoiseGenerator::new(world.seed); 
        let height = noise_gen.get_height(eye_bp.x, eye_bp.z);
        let biome = noise_gen.get_biome_at(eye_bp.x, eye_bp.z, height);
        let fog_color = match biome {
            "swamp" => [0.3, 0.4, 0.2, 1.0],
            "desert" => [0.8, 0.7, 0.5, 1.0],
            "ice_plains" => [0.9, 0.9, 1.0, 1.0],
            _ => [0.5, 0.8, 0.9, 1.0], // Default sky blue
        };

        self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[fog_color[0], fog_color[1], fog_color[2], fog_color[3], time, is_underwater, 0.0, 0.0]));

        let mut ent_v = Vec::new(); let mut ent_i = Vec::new(); let mut ent_off = 0;
        for rp in &world.remote_players {
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y, rp.position.z], rp.rotation, -0.3, 0.0, -0.3, 0.6, f, 13); }
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y+0.65, rp.position.z], rp.rotation, -0.3, 0.0, -0.3, 0.6, f, 13); }
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y+1.3, rp.position.z], rp.rotation, -0.25, 0.0, -0.25, 0.5, f, 13); }
        }
        for e in &world.entities {
            let (t, _, _) = e.item_type.get_texture_indices();
            let rot = time * 1.5 + e.bob_offset; let by = ((time * 4.0 + e.bob_offset).sin() * 0.05) + 0.12; // Toned down shake
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [e.position.x, e.position.y+by, e.position.z], rot, -0.125, -0.125, -0.125, 0.25, f, t); }
        }
        if !ent_v.is_empty() {
            self.entity_vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Ent VB"), contents: bytemuck::cast_slice(&ent_v), usage: BufferUsages::VERTEX });
            self.entity_index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Ent IB"), contents: bytemuck::cast_slice(&ent_i), usage: BufferUsages::INDEX });
        }

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Encoder") });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor { label: Some("3D Pass"), color_attachments: &[Some(RenderPassColorAttachment { view: &view, resolve_target: None, ops: Operations { load: LoadOp::Clear(Color { r: 0.5, g: 0.8, b: 0.9, a: 1.0 }), store: StoreOp::Store } })], depth_stencil_attachment: Some(RenderPassDepthStencilAttachment { view: &self.depth_texture, depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }), stencil_ops: None }), timestamp_writes: None, occlusion_query_set: None });
            pass.set_pipeline(&self.pipeline); pass.set_bind_group(0, &self.bind_group, &[]); pass.set_bind_group(1, &self.camera_bind_group, &[]); pass.set_bind_group(2, &self.time_bind_group, &[]);
for m in self.chunk_meshes.values() { pass.set_vertex_buffer(0, m.vertex_buffer.slice(..)); pass.set_index_buffer(m.index_buffer.slice(..), IndexFormat::Uint32); pass.draw_indexed(0..m.index_count, 0, 0..1); }
            if !ent_v.is_empty() { pass.set_vertex_buffer(0, self.entity_vertex_buffer.slice(..)); pass.set_index_buffer(self.entity_index_buffer.slice(..), IndexFormat::Uint32); pass.draw_indexed(0..ent_i.len() as u32, 0, 0..1); }
            
// --- BREAKING CRACKS OVERLAY (STUB) ---
            if self.break_progress > 0.0 {
                // Future crack logic
            }
        }

// UI
        let mut uv = Vec::new(); let mut ui = Vec::new(); let mut uoff = 0;
        let aspect = self.config.width as f32 / self.config.height as f32;

        // DIABOLICAL LAYOUT CONSTANTS (Defined early for scope access)
        let sw = 0.12; 
        let sh = sw * aspect; 
        let sx = -(sw * 9.0) / 2.0; 
        let by = -0.9;
        
        if !player.inventory_open && !is_paused { 
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -0.015, -0.015 * aspect, 0.03, 0.03 * aspect, 240); 
        }

        // --- DRAW AIR BUBBLES ---
        const UI_BUBBLE: u32 = 243;
        if player.air < player.max_air {
            let bubble_count = (player.air / player.max_air * 10.0).ceil() as i32;
            let bx_bubbles = sx + sw * 5.0; 
            let by_bubbles = by + sh + 0.08 * aspect;
            for i in 0..10 {
                if i < bubble_count {
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, bx_bubbles + i as f32 * 0.045, by_bubbles, 0.04, 0.04 * aspect, UI_BUBBLE);
                }
            }
        }

        // --- DRAW HOTBAR ---
        
        if player.inventory_open {
             self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -1.0, -1.0, 2.0, 2.0, 240); // Dim BG
             self.draw_text("INVENTORY", -0.2, 0.8, 0.08, &mut uv, &mut ui, &mut uoff);
        }

        if !is_paused || player.inventory_open {
            for i in 0..9 {
                let x = sx + (i as f32 * sw);
                if i == player.inventory.selected_hotbar_slot { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x-0.005, by-0.005*aspect, sw+0.01, sh+0.01*aspect, 241); }
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x, by, sw, sh, 240);
if let Some(stack) = &player.inventory.slots[i] {
                    let (t, _, _) = stack.item.get_texture_indices();
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x+0.02, by+0.02*aspect, sw-0.04, sh-0.04*aspect, t);
                    // Shifted text so it doesn't overlap slots
if stack.count > 1 { self.draw_text(&format!("{}", stack.count), x + 0.07, by + 0.02, 0.04, &mut uv, &mut ui, &mut uoff); }
                    
                    // DURABILITY BAR
                    if stack.item.is_tool() {
                        let max_dur = stack.item.get_max_durability();
                        if stack.durability < max_dur {
                            let ratio = stack.durability as f32 / max_dur as f32;
                            let bar_w = sw * 0.7 * ratio;
                            let bar_x = x + (sw * 0.15);
                            let bar_y = by + 0.05 * aspect;
                            // Color logic: Green -> Yellow -> Red
                            let tex = if ratio > 0.5 { 244 } else if ratio > 0.2 { 245 } else { 246 }; // Use different UI bar colors
                            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, bar_x, bar_y, bar_w, 0.01 * aspect, tex);
                        }
                    }
                }
            }
            if !player.inventory_open {
                for i in 0..10 { if player.health > (i as f32)*2.0 { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, sx + i as f32 * 0.05, by+sh+0.02*aspect, 0.045, 0.045*aspect, 242); } }
                if self.break_progress > 0.0 { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -0.1, -0.1, 0.2 * self.break_progress, 0.02*aspect, 244); }
            }
        }

        if player.inventory_open {
            let iby = by + sh * 1.5;
            // Main Grid
            for r in 0..3 { for c in 0..9 {
                let idx = 9 + r * 9 + c; let x = sx + c as f32 * sw; let y = iby + r as f32 * sh;
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x, y, sw, sh, 240);
                if let Some(stack) = &player.inventory.slots[idx] { 
                    let (t, _, _) = stack.item.get_texture_indices(); 
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x+0.02, y+0.02*aspect, sw-0.04, sh-0.04*aspect, t); 
                    if stack.count > 1 { self.draw_text(&format!("{}", stack.count), x+0.01, y+0.01, 0.03, &mut uv, &mut ui, &mut uoff); } 
                }
            }}
            // Crafting
            let cx = 0.3; let cy = 0.5;
            self.draw_text(if player.crafting_open { "CRAFTING TABLE" } else { "CRAFTING" }, 0.3, 0.7, 0.05, &mut uv, &mut ui, &mut uoff);
            let grid_size = if player.crafting_open { 3 } else { 2 };
            for r in 0..grid_size { for c in 0..grid_size {
                let x = cx + c as f32 * sw; let y = cy - r as f32 * sh;
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x, y, sw, sh, 240);
                let idx = if player.crafting_open { r*3+c } else { match r*2+c { 0=>0, 1=>1, 2=>3, 3=>4, _=>0 } };
                if let Some(stack) = &player.inventory.crafting_grid[idx] { 
                    let (t, _, _) = stack.item.get_texture_indices(); 
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x+0.02, y+0.02*aspect, sw-0.04, sh-0.04*aspect, t); 
                    if stack.count > 1 { self.draw_text(&format!("{}", stack.count), x+0.01, y+0.01, 0.03, &mut uv, &mut ui, &mut uoff); }
                }
            }}
            // Output
            let ox = cx + 3.0*sw; let oy = cy - 0.5*sh;
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ox, oy, sw, sh, 240); 
            self.draw_text("->", cx + 2.1*sw, oy+0.05, 0.04, &mut uv, &mut ui, &mut uoff);
            if let Some(stack) = &player.inventory.crafting_output { 
                let (t, _, _) = stack.item.get_texture_indices(); 
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ox+0.02, oy+0.02*aspect, sw-0.04, sh-0.04*aspect, t); 
                if stack.count > 1 { self.draw_text(&format!("{}", stack.count), ox+0.01, oy+0.01, 0.03, &mut uv, &mut ui, &mut uoff); } 
            }
            // Render Held Item & Tooltip
            let (mx, my) = cursor_pos; let ndc_x = (mx as f32 / self.config.width as f32)*2.0-1.0; let ndc_y = -((my as f32 / self.config.height as f32)*2.0-1.0);
            if let Some(stack) = &player.inventory.cursor_item {
                let (t, _, _) = stack.item.get_texture_indices();
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ndc_x - sw/2.0, ndc_y - sh/2.0, sw, sh, t);
                if stack.count > 1 { self.draw_text(&format!("{}", stack.count), ndc_x - sw/2.0, ndc_y - sh/2.0, 0.03, &mut uv, &mut ui, &mut uoff); }
            } else {
                // Tooltip
                for i in 0..9 {
                    let x = sx + (i as f32 * sw); 
                    if ndc_x >= x && ndc_x < x+sw && ndc_y >= by && ndc_y < by+sh {
                        if let Some(s) = &player.inventory.slots[i] {
                            self.draw_text(s.item.get_display_name(), ndc_x + 0.02, ndc_y - 0.02, 0.025, &mut uv, &mut ui, &mut uoff);
                        }
                    }
                }
            }
        }

        if !uv.is_empty() {
            let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("UI VB"), contents: bytemuck::cast_slice(&uv), usage: BufferUsages::VERTEX });
            let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("UI IB"), contents: bytemuck::cast_slice(&ui), usage: BufferUsages::INDEX });
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor { label: Some("UI Pass"), color_attachments: &[Some(RenderPassColorAttachment { view: &view, resolve_target: None, ops: Operations { load: LoadOp::Load, store: StoreOp::Store } })], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None });
            pass.set_pipeline(&self.ui_pipeline); pass.set_bind_group(0, &self.bind_group, &[]); pass.set_vertex_buffer(0, vb.slice(..)); pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32); pass.draw_indexed(0..ui.len() as u32, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
output.present();
Ok(())
}
}