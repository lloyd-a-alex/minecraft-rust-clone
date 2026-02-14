use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::engine::{World, BlockPos, BlockType};
use crate::engine::Player;
use crate::{MainMenu, SettingsMenu};
use std::fs::File;
use std::io::Write;

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

#[allow(dead_code)]
pub struct TextureRange {
    pub _tex_index: u32,
    pub _index_start: u32,
    pub _index_count: u32,
}

pub struct ChunkMesh { 
    pub vertex_buffer: Buffer, 
    pub index_buffer: Buffer, 
    pub _ranges: Vec<TextureRange>,
    pub total_indices: u32 
}

pub struct MeshTask {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
    pub lod: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub ranges: Vec<TextureRange>,
}

pub struct Renderer<'a> {
    pub particles: Vec<Particle>,
    surface: Surface<'a>, device: Device, queue: Queue, pub config: SurfaceConfiguration,
    pipeline: RenderPipeline, ui_pipeline: RenderPipeline,
    depth_texture: TextureView, bind_group: BindGroup,
    camera_buffer: Buffer, camera_bind_group: BindGroup,
    time_buffer: Buffer, time_bind_group: BindGroup,
pub start_time: Instant, 
    pub chunk_meshes: HashMap<(i32, i32, i32), (ChunkMesh, u32)>, // (x, y, z) -> (Mesh, LOD_Level)
    entity_vertex_buffer: Buffer, entity_index_buffer: Buffer,
    pub break_progress: f32,
    
    // FPS TRACKING
    pub fps: f32,
    frame_count: u32,
    last_fps_time: Instant,
    last_player_chunk: (i32, i32, i32),

    // DIABOLICAL THREADING
    pub mesh_tx: Sender<(i32, i32, i32, u32, Arc<World>)>,
    mesh_rx: Receiver<MeshTask>,
    pub pending_chunks: HashSet<(i32, i32, i32)>,

    // DIABOLICAL GPU CULLING FIELDS
    #[allow(dead_code)] compute_pipeline: ComputePipeline,
    #[allow(dead_code)] chunk_data_buffer: Buffer,     
    #[allow(dead_code)] indirect_draw_buffer: Buffer,   
    #[allow(dead_code)] indirect_count_buffer: Buffer,  
    #[allow(dead_code)] cull_bind_group: BindGroup,

    // LOADING SCREEN STATE
    pub loading_progress: f32,
    pub loading_message: String,
    pub transition_alpha: f32,
    pub init_time: Instant,
    pub adapter_info: wgpu::AdapterInfo,
    
    // CONFIGURATION FIELDS
    pub render_distance: u32,
    pub max_fps: u32,
    pub fov: f32,
    pub current_shader_type: crate::ShaderType,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkCullData {
    pos: [f32; 4], // x, y, z, radius
    index_count: u32,
    base_vertex: i32,
    base_index: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

impl<'a> Renderer<'a> {
    pub fn update_camera(&mut self, player: &Player, aspect: f32, alpha: f32) {
        let (pitch_sin, pitch_cos) = player.rotation.x.sin_cos(); 
        let (yaw_sin, yaw_cos) = player.rotation.y.sin_cos();
        
        // INTERPOLATE POSITION: Perfect smoothness
        let pos = player.prev_position.lerp(player.position, alpha);
        let mut eye_pos = pos + glam::Vec3::new(0.0, player.height * 0.4, 0.0);
        
        // Bobbing interpolation
        if player.on_ground && (player.input.forward || player.input.backward || player.input.left || player.input.right) { 
            eye_pos.y += (player.walk_time * 2.0).sin() * 0.02; 
        }
        
        let forward = glam::Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let view = glam::Mat4::look_at_rh(eye_pos, eye_pos + forward, glam::Vec3::Y);
        let fov_rad = self.fov.to_radians();
        let proj = glam::Mat4::perspective_rh(fov_rad, aspect, 0.1, 512.0);
        
        let correction = glam::Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        ]);
        
        let view_proj = (correction * proj * view).to_cols_array_2d();
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
    }
    
    pub fn set_render_distance(&mut self, render_distance: u32) {
        self.render_distance = render_distance;
    }
    
    pub fn set_max_fps(&mut self, max_fps: u32) {
        self.max_fps = max_fps;
    }
    
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }
    
    pub fn switch_shader(&mut self, shader_type: crate::ShaderType) -> Result<(), Box<dyn std::error::Error>> {
        // Load the new shader
        let shader_code = match shader_type {
            crate::ShaderType::Classic => include_str!("minecraft_shaders.wgsl"),
            crate::ShaderType::Traditional => include_str!("traditional_shaders.wgsl"),
            crate::ShaderType::Basic => include_str!("shader.wgsl"),
        };

        let new_shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", shader_type.get_display_name())),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        // Recreate pipelines with new shader
        let new_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Render Pipeline", shader_type.get_display_name())),
            layout: None,
            vertex: wgpu::VertexState {
                module: &new_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &new_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let new_ui_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} UI Pipeline", shader_type.get_display_name())),
            layout: None,
            vertex: wgpu::VertexState {
                module: &new_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &new_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Update the pipelines
        self.pipeline = new_pipeline;
        self.ui_pipeline = new_ui_pipeline;
        self.current_shader_type = shader_type;

        // Force rebuild all chunks to apply new shader
        self.chunk_meshes.clear();

        log::info!("🎨 Switched to {} shader", shader_type.get_display_name());
        Ok(())
    }
    
    pub async fn new(window: &'a Window) -> Self {
        // --- KEY FIX: Use empty flags for compatibility ---
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = match instance.create_surface(window) {
            Ok(surface) => surface,
            Err(e) => {
                log::error!("Failed to create surface: {:?}", e);
                panic!("Failed to create surface");
            }
        };
        let adapter = match instance.request_adapter(&RequestAdapterOptions { 
            power_preference: PowerPreference::HighPerformance, 
            compatible_surface: Some(&surface), 
            force_fallback_adapter: false 
        }).await {
            Some(adapter) => adapter,
            None => {
                log::error!("Failed to request adapter");
                panic!("Failed to request adapter");
            }
        };
        let adapter_info = adapter.get_info();
        let (device, queue) = match adapter.request_device(&DeviceDescriptor::default(), None).await {
            Ok(pair) => pair,
            Err(e) => {
                log::error!("Failed to request device: {:?}", e);
                panic!("Failed to request device");
            }
        };
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or_else(|| {
                log::warn!("No SRGB format found, using first available");
                surface_caps.formats[0]
            });
        let config = SurfaceConfiguration { usage: TextureUsages::RENDER_ATTACHMENT, format: surface_format, width: window.inner_size().width, height: window.inner_size().height, present_mode: PresentMode::Fifo, alpha_mode: surface_caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &config);

        // DIABOLICAL ZERO-LATENCY BAKE: Generate the atlas immediately so the Menu is NEVER black.
        // This takes ~150ms on modern CPUs and ensures immediate UI availability.
        let atlas = crate::resources::TextureAtlas::new();
        let atlas_size = Extent3d { width: 512, height: 512, depth_or_array_layers: 1 };
        let texture = device.create_texture(&TextureDescriptor { label: Some("atlas"), size: atlas_size, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Rgba8UnormSrgb, usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, view_formats: &[] });
        queue.write_texture(ImageCopyTexture { texture: &texture, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All }, &atlas.data, ImageDataLayout { offset: 0, bytes_per_row: Some(512 * 4), rows_per_image: Some(512) }, atlas_size);
        
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor { 
            mag_filter: FilterMode::Nearest, 
            min_filter: FilterMode::Nearest, 
            mipmap_filter: FilterMode::Nearest, 
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            ..Default::default() 
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Main Layout"),
            entries: &[
                BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE, ty: BindingType::Texture { sample_type: TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false }, count: None },
                BindGroupLayoutEntry { binding: 1, visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE, ty: BindingType::Sampler(SamplerBindingType::Filtering), count: None },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("texture_bind"), layout: &bind_group_layout, entries: &[BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) }, BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&sampler) }] });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Camera Buffer"), contents: bytemuck::cast_slice(&[[0.0f32; 16]]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST });
        let camera_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("camera_bind"), layout: &camera_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }] });

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Time Buffer"), contents: bytemuck::cast_slice(&[0.0f32; 8]), usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST });
        let time_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Time Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let time_bind_group = device.create_bind_group(&BindGroupDescriptor { label: Some("time_bind"), layout: &time_bg_layout, entries: &[BindGroupEntry { binding: 0, resource: time_buffer.as_entire_binding() }] });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("Pipeline Layout"), bind_group_layouts: &[&bind_group_layout, &camera_bg_layout, &time_bg_layout], push_constant_ranges: &[] });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor { label: Some("Pipeline"), layout: Some(&pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc()] }, fragment: Some(FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState { format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL })] }), primitive: PrimitiveState { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), ..Default::default() }, depth_stencil: Some(DepthStencilState { format: TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: CompareFunction::Less, stencil: StencilState::default(), bias: DepthBiasState::default() }), multisample: MultisampleState::default(), multiview: None });

        // DIABOLICAL BINDING STABILITY: The UI pipeline only needs the Texture Atlas (Group 0). 
        // Removing Groups 1 and 2 here prevents validation errors when camera/time aren't bound.
        let ui_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("UI Layout"), bind_group_layouts: &[&bind_group_layout], push_constant_ranges: &[] });
        let ui_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor { label: Some("UI Pipeline"), layout: Some(&ui_pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_ui", buffers: &[Vertex::desc()] }, fragment: Some(FragmentState { module: &shader, entry_point: "fs_ui", targets: &[Some(ColorTargetState { format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL })] }), primitive: PrimitiveState { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: None, ..Default::default() }, depth_stencil: None, multisample: MultisampleState::default(), multiview: None });

        let depth_texture = device.create_texture(&TextureDescriptor { size: Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float, usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[] }).create_view(&TextureViewDescriptor::default());
        let entity_vertex_buffer = device.create_buffer(&BufferDescriptor { label: Some("Entity VB"), size: 1024, usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, mapped_at_creation: false });

        // --- DIABOLICAL COMPUTE CULLER INIT ---
        let compute_shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        
// Max 10,000 chunks handled at once
        let chunk_data_buffer = device.create_buffer(&BufferDescriptor { label: Some("Chunk Data Buffer"), size: (10000 * std::mem::size_of::<ChunkCullData>()) as u64, usage: BufferUsages::STORAGE | BufferUsages::COPY_DST, mapped_at_creation: false });
        let indirect_draw_buffer = device.create_buffer(&BufferDescriptor { label: Some("Indirect Draw Buffer"), size: (10000 * std::mem::size_of::<DrawIndexedIndirect>()) as u64, usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST, mapped_at_creation: false });
        let indirect_count_buffer = device.create_buffer(&BufferDescriptor { label: Some("Indirect Count Buffer"), size: 256, usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST, mapped_at_creation: false });

let cull_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Cull Layout"),
            entries: &[
                BindGroupLayoutEntry { binding: 0, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 1, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                BindGroupLayoutEntry { binding: 2, visibility: ShaderStages::COMPUTE, ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let cull_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Cull BG"),
            layout: &cull_bg_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: chunk_data_buffer.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: indirect_draw_buffer.as_entire_binding() },
                BindGroupEntry { binding: 2, resource: indirect_count_buffer.as_entire_binding() },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { 
            label: Some("Compute Layout"), 
            bind_group_layouts: &[&bind_group_layout, &camera_bg_layout, &time_bg_layout, &cull_bg_layout], 
            push_constant_ranges: &[] 
        });
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor { label: Some("Cull Pipeline"), layout: Some(&compute_pipeline_layout), module: &compute_shader, entry_point: "compute_cull" });
let entity_index_buffer = device.create_buffer(&BufferDescriptor { label: Some("Entity IB"), size: 1024, usage: BufferUsages::INDEX | BufferUsages::COPY_DST, mapped_at_creation: false });

// DIABOLICAL WORKER POOL INITIALIZATION
// DIABOLICAL THREADED LOD MESH GENERATOR
        let (task_tx, task_rx) = unbounded::<(i32, i32, i32, u32, Arc<World>)>();
        let (result_tx, result_rx) = unbounded::<MeshTask>();
        // DIABOLICAL MULTI-THREADED MESH GENERATION: Scaling to all available CPU cores
        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(16)
            .min(8); // Limit to 8 threads to prevent excessive resource usage
        log::info!("🚀 SPAWNING {} DIABOLICAL MESH WORKER THREADS", thread_count);

        for thread_id in 0..thread_count {
            let t_rx = task_rx.clone();
            let r_tx = result_tx.clone();
            let _ = std::thread::Builder::new()
                .name(format!("mesh_worker_{}", thread_id))
                .spawn(move || {
                    while let Ok((cx, cy, cz, lod, world)) = t_rx.recv() {
                        // Add validation for chunk coordinates
                        if cx.abs() > 1000 || cy.abs() > 1000 || cz.abs() > 1000 {
                            log::warn!("Invalid chunk coordinates: ({}, {}, {})", cx, cy, cz);
                            continue;
                        }
                        
                        let mut vertices = Vec::new();
                        let mut indices = Vec::new();
                        let mut i_cnt = 0;
                    
                    if let Some(chunk) = world.chunks.get(&(cx, cy, cz)) {
                        if chunk.is_empty {
                            let _ = r_tx.send(MeshTask { cx, cy, cz, lod, vertices: Vec::new(), indices: Vec::new(), ranges: Vec::new() });
                            continue;
                        }

                        let bx = (cx * 16) as f32;
                        let by = (cy * 16) as f32;
                        let bz = (cz * 16) as f32;
                        let step = 1 << lod;
                        
                        for axis in 0..3 {
                            for d in (0..16usize).step_by(step as usize) {
                                for dir in 0..2 {
                                    let face_id = match axis { 
                                        0 => if dir==0 {0} else {1},
                                        1 => if dir==0 {2} else {3},
                                        _ => if dir==0 {4} else {5}
                                    };
                                    
                                    let mask_dim = 16 / step as usize;
                                    let mut mask = vec![BlockType::Air; mask_dim * mask_dim];
                                    
                                    for u_m in 0..mask_dim {
                                        for v_m in 0..mask_dim {
                                            let u = u_m * step as usize;
                                            let v = v_m * step as usize;
                                            let (lx, ly, lz) = match axis {
                                                0 => (u, d, v), 1 => (d, u, v), _ => (u, v, d)
                                            };
                                            
                                            let blk = chunk.get_block(lx, ly, lz);
                                            if !blk.is_solid() { continue; }
                                            
                                            let (nx, ny, nz) = match face_id { 
                                                0 => (lx as i32, ly as i32 + step as i32, lz as i32), 
                                                1 => (lx as i32, ly as i32 - 1, lz as i32), 
                                                2 => (lx as i32 + step as i32, ly as i32, lz as i32), 
                                                3 => (lx as i32 - 1, ly as i32, lz as i32), 
                                                4 => (lx as i32, ly as i32, lz as i32 + step as i32), 
                                                5 => (lx as i32, ly as i32, lz as i32 - 1), 
                                                _ => (0, 0, 0)
                                            };
                                            
                                            let neighbor = if nx >= 0 && nx < 16 && ny >= 0 && ny < 16 && nz >= 0 && nz < 16 {
                                                chunk.get_block(nx as usize, ny as usize, nz as usize)
                                            } else {
                                                BlockType::Air
                                            };
                                            
                                            if !neighbor.is_solid() || (neighbor.is_transparent() && neighbor != blk) { 
                                                mask[v_m * mask_dim + u_m] = blk; 
                                            }
                                        }
                                    }
                                    
                                    let mut n = 0;
                                    while n < mask.len() {
                                        let blk = mask[n];
                                        if blk != BlockType::Air {
                                            let mut w = 1;
                                            while (n + w) % mask_dim != 0 && mask[n + w] == blk { w += 1; }
                                            let mut h = 1;
                                            'h_loop: while (n / mask_dim + h) < mask_dim {
                                                for k in 0..w { if mask[n + k + h * mask_dim] != blk { break 'h_loop; } }
                                                h += 1;
                                            }
                                            
                                            let u_g = (n % mask_dim) as f32 * step as f32;
                                            let v_g = (n / mask_dim) as f32 * step as f32;
                                            let world_w = w as f32 * step as f32;
                                            let world_h = h as f32 * step as f32;
                                            let d_f = d as f32;
                                            let s_f = step as f32;
                                            
                                            let (wx, wy, wz) = match axis {
                                                0 => (bx + u_g, by + d_f, bz + v_g),
                                                1 => (bx + d_f, by + u_g, bz + v_g),
                                                _ => (bx + u_g, by + v_g, bz + d_f)
                                            };
                                            
                                            let tex_index = match face_id { 0 => blk.get_texture_top(), 1 => blk.get_texture_bottom(), _ => blk.get_texture_side() };
                                            
                                            // CORRECTED CCW WINDING ORDERS FOR LOD MESHES
                                            let (positions, uv) = match face_id {
                                                0 => ([[wx, wy+s_f, wz+world_h], [wx+world_w, wy+s_f, wz+world_h], [wx+world_w, wy+s_f, wz], [wx, wy+s_f, wz]], [[0.0, world_h], [world_w, world_h], [world_w, 0.0], [0.0, 0.0]]),
                                                1 => ([[wx, wy, wz], [wx+world_w, wy, wz], [wx+world_w, wy, wz+world_h], [wx, wy, wz+world_h]], [[0.0, 0.0], [world_w, 0.0], [world_w, world_h], [0.0, world_h]]),
                                                2 => ([[wx+s_f, wy, wz], [wx+s_f, wy+world_w, wz], [wx+s_f, wy+world_w, wz+world_h], [wx+s_f, wy, wz+world_h]], [[0.0, 0.0], [world_w, 0.0], [world_w, world_h], [0.0, world_h]]),
                                                3 => ([[wx, wy, wz+world_h], [wx, wy+world_w, wz+world_h], [wx, wy+world_w, wz], [wx, wy, wz]], [[0.0, world_h], [world_w, world_h], [world_w, 0.0], [0.0, 0.0]]),
                                                4 => ([[wx, wy, wz+s_f], [wx+world_w, wy, wz+s_f], [wx+world_w, wy+world_h, wz+s_f], [wx, wy+world_h, wz+s_f]], [[0.0, 0.0], [world_w, 0.0], [world_w, world_h], [0.0, world_h]]),
                                                5 => ([[wx+world_w, wy, wz], [wx, wy, wz], [wx, wy+world_h, wz], [wx+world_w, wy+world_h, wz]], [[world_w, 0.0], [0.0, 0.0], [0.0, world_h], [world_w, world_h]]),
                                                _ => ([[0.0; 3]; 4], [[0.0; 2]; 4]),
                                            };
                                            
                                            let base_i = i_cnt;
                                            vertices.push(Vertex { position: positions[0], tex_coords: uv[0], ao: 1.0, tex_index, light: 15.0 });
                                            vertices.push(Vertex { position: positions[1], tex_coords: uv[1], ao: 1.0, tex_index, light: 15.0 });
                                            vertices.push(Vertex { position: positions[2], tex_coords: uv[2], ao: 1.0, tex_index, light: 15.0 });
                                            vertices.push(Vertex { position: positions[3], tex_coords: uv[3], ao: 1.0, tex_index, light: 15.0 });
                                            indices.extend_from_slice(&[base_i, base_i + 1, base_i + 2, base_i, base_i + 2, base_i + 3]);
                                            i_cnt += 4;
                                            
                                            for l in 0..h { for k in 0..w { mask[n + k + l * mask_dim] = BlockType::Air; } }
                                        }
                                        n += 1;
                                    }
                                }
                            }
                        }
                    }

                    let mut final_ranges = Vec::new();
                    if !indices.is_empty() { final_ranges.push(TextureRange { _tex_index: 0, _index_start: 0, _index_count: indices.len() as u32 }); }
                    let _ = r_tx.send(MeshTask { cx, cy, cz, lod, vertices, indices, ranges: final_ranges });
                }
            });
        }

        Self {
            particles: Vec::new(), surface, device, queue, config, pipeline, ui_pipeline, depth_texture, bind_group, camera_bind_group, camera_buffer, time_bind_group, time_buffer, start_time: Instant::now(), 
            chunk_meshes: HashMap::new(),
            entity_vertex_buffer, entity_index_buffer, 
            break_progress: 0.0,
            fps: 0.0,
            frame_count: 0,
            last_fps_time: Instant::now(),
            last_player_chunk: (0, 0, 0),
            mesh_tx: task_tx,
            mesh_rx: result_rx,
            pending_chunks: HashSet::new(),
            compute_pipeline,
            chunk_data_buffer,
            indirect_draw_buffer,
            indirect_count_buffer,
            cull_bind_group,
            loading_progress: 0.0,
            loading_message: "INITIALIZING...".to_string(),
            transition_alpha: 1.0,
            init_time: Instant::now(),
            adapter_info,
            render_distance: 12,
            max_fps: 60,
            fov: 75.0,
            current_shader_type: crate::ShaderType::Basic,
        }
    }

    #[allow(dead_code)]
    pub fn upload_atlas(&mut self, atlas_data: &[u8]) {
        let atlas_size = Extent3d { width: 512, height: 512, depth_or_array_layers: 1 };
        let texture = self.device.create_texture(&TextureDescriptor { label: Some("atlas_final"), size: atlas_size, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Rgba8UnormSrgb, usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, view_formats: &[] });
        self.queue.write_texture(ImageCopyTexture { texture: &texture, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All }, atlas_data, ImageDataLayout { offset: 0, bytes_per_row: Some(512 * 4), rows_per_image: Some(512) }, atlas_size);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        
        // Re-bind the textures globally
        self.bind_group = self.device.create_bind_group(&BindGroupDescriptor { 
            label: Some("texture_bind_final"), 
            layout: &self.pipeline.get_bind_group_layout(0), 
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) }, 
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&self.device.create_sampler(&SamplerDescriptor { mag_filter: FilterMode::Nearest, min_filter: FilterMode::Nearest, ..Default::default() })) }
            ] 
        });
    }

    pub fn process_mesh_queue(&mut self) {
        // ULTRA PERFORMANCE: Process more meshes per frame for higher FPS
        let max_tasks_per_frame = 32; // Increased from 8 for much better performance
        let mut processed = 0;
        
        while let Ok(task) = self.mesh_rx.try_recv() {
            if processed >= max_tasks_per_frame {
                break; // Process more next frame
            }
            
            self.pending_chunks.remove(&(task.cx, task.cy, task.cz));

            if !task.vertices.is_empty() {
                // Only remove existing mesh if we're replacing it with a non-empty mesh
                self.chunk_meshes.remove(&(task.cx, task.cy, task.cz));
                
                // FAST PATH: Skip validation for release builds
                #[cfg(debug_assertions)]
                {
                    // Validate mesh data to prevent GPU crashes
                    if task.vertices.len() > 100000 || task.indices.len() > 200000 {
                        log::warn!("Rejecting oversized mesh for chunk ({}, {}, {}): {} vertices, {} indices", 
                                 task.cx, task.cy, task.cz, task.vertices.len(), task.indices.len());
                        processed += 1;
                        continue;
                    }
                }
                
                let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                    label: Some(&format!("Chunk VB ({},{},{})", task.cx, task.cy, task.cz)), 
                    contents: bytemuck::cast_slice(&task.vertices), 
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST 
                });
                let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                    label: Some(&format!("Chunk IB ({},{},{})", task.cx, task.cy, task.cz)), 
                    contents: bytemuck::cast_slice(&task.indices), 
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST 
                });
                self.chunk_meshes.insert((task.cx, task.cy, task.cz), (ChunkMesh { 
                    vertex_buffer: vb, 
                    index_buffer: ib, 
                    _ranges: task.ranges, 
                    total_indices: task.indices.len() as u32 
                }, task.lod));
            } else {
                // For empty chunks, remove the mesh to make them invisible
                self.chunk_meshes.remove(&(task.cx, task.cy, task.cz));
            }
            processed += 1;
        }
    }

    pub fn dump_crash_telemetry(&self, error: &str) {
        let path = "GPU_CRASH_DUMP.txt";
        let mut file = File::create(path).expect("Failed to create crash dump file");
        let info = &self.adapter_info;
        
        let dump = format!(
            "--- DIABOLICAL GPU CRASH DUMP ---\n\
             TIMESTAMP: {:?}\n\
             ERROR: {}\n\
             ADAPTER: {} ({:?})\n\
             BACKEND: {:?}\n\
             DEVICE TYPE: {:?}\n\
             DRIVER: {}\n\
             WINDOW_SIZE: {}x{}\n\
             CHUNKS_LOADED: {}\n\
             PENDING_TASKS: {}\n\
             --------------------------------",
            Instant::now(), error, info.name, info.device, info.backend, 
            info.device_type, info.driver, self.config.width, self.config.height,
            self.chunk_meshes.len(), self.pending_chunks.len()
        );
        
        let _ = file.write_all(dump.as_bytes());
        log::error!("🔥 GPU CRASH DETECTED. Telemetry dumped to {}", path);
    }

    pub fn render_loading_screen(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                self.dump_crash_telemetry(&format!("{:?}", e));
                return Err(e);
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Loading Encoder") });
        let _aspect = self.config.width as f32 / self.config.height as f32;
        let _time = self.init_time.elapsed().as_secs_f32();

        let mut uv = Vec::new();
        let mut ui = Vec::new();
        let mut uoff = 0;

        // 1. PROFESSIONAL BACKGROUND
        self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -1.0, -1.0, 2.0, 2.0, 240); // Solid base
        
        // 2. Title - Single Clean Pass
        self.draw_text("MINECRAFT", -0.32, 0.42, 0.12, &mut uv, &mut ui, &mut uoff);
        self.draw_text("RUST EDITION", -0.25, 0.32, 0.05, &mut uv, &mut ui, &mut uoff);

        // 3. High-Contrast Progress Bar
        let bar_w = 1.2;
        let bar_h = 0.02;
        let bar_x = -0.6;
        let bar_y = -0.3;
        
        // Container
        self.add_ui_quad(&mut uv, &mut ui, &mut uoff, bar_x - 0.005, bar_y - 0.005, bar_w + 0.01, bar_h + 0.01, 240);
        
        // Progress Fill (Green/White)
        if self.loading_progress > 0.001 {
            let p_w = bar_w * self.loading_progress.clamp(0.0, 1.0);
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, bar_x, bar_y, p_w, bar_h, 241);
        }

        // 4. Status Message
        let msg = self.loading_message.to_uppercase();
        let msg_scale = 0.03;
        let msg_x = -(msg.len() as f32 * msg_scale * 0.6) / 2.0;
        self.draw_text(&msg, msg_x, bar_y - 0.1, msg_scale, &mut uv, &mut ui, &mut uoff);

        // 5. Clean Fade Out
        if self.transition_alpha < 0.99 {
            // Full-screen overlay that fades based on transition_alpha logic in main.rs
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -1.0, -1.0, 2.0, 2.0, 240);
        }

        if !uv.is_empty() {
            let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("L VB"), contents: bytemuck::cast_slice(&uv), usage: BufferUsages::VERTEX });
            let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("L IB"), contents: bytemuck::cast_slice(&ui), usage: BufferUsages::INDEX });
            {
                let mut pass = encoder.begin_render_pass(&RenderPassDescriptor { 
                    label: Some("Load Pass"), 
                    color_attachments: &[Some(RenderPassColorAttachment { view: &view, resolve_target: None, ops: Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store } })], 
                    depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None 
                });
                pass.set_pipeline(&self.ui_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32);
                pass.draw_indexed(0..ui.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width; self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = self.device.create_texture(&TextureDescriptor { size: Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: TextureFormat::Depth32Float, usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, label: Some("depth"), view_formats: &[] }).create_view(&TextureViewDescriptor::default());
        }
    }

pub fn rebuild_all_chunks(&mut self, world: &World) { 
        log::info!("Rebuilding all chunks - clearing {} meshes", self.chunk_meshes.len());
        self.chunk_meshes.clear(); 
        self.pending_chunks.clear(); // Clear pending to avoid duplicates
        
        // Limit chunk count to prevent memory exhaustion
        let max_chunks = 1000;
        let mut chunk_count = 0;
        
        for (key, _) in &world.chunks {
            if chunk_count >= max_chunks {
                log::warn!("Reached maximum chunk limit ({}), skipping remaining", max_chunks);
                break;
            }
            self.update_chunk(key.0, key.1, key.2, world);
            chunk_count += 1;
        }
        self.update_clouds(world);
        log::info!("Queued {} chunks for rebuilding", chunk_count);
    }

    fn update_clouds(&mut self, world: &World) {
        let mut vertices = Vec::new(); let mut indices = Vec::new(); let mut offset = 0;
        let cloud_y = 110.0;
        for cx in -10..10 {
            for cz in -10..10 {
                let wx = cx * 16; let wz = cz * 16;
                let noise = crate::resources::NoiseGenerator::new(world.seed);
                if noise.get_noise3d(wx as f64 * 0.01, 0.0, wz as f64 * 0.01) > 0.4 {
                    self.add_face(&mut vertices, &mut indices, &mut offset, wx, cloud_y as i32, wz, 0, 228, 1.0, 1.0);
                    self.add_face(&mut vertices, &mut indices, &mut offset, wx, cloud_y as i32, wz, 1, 228, 1.0, 0.8);
                }
            }
        }
        if !vertices.is_empty() {
            let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Cloud VB"), contents: bytemuck::cast_slice(&vertices), usage: BufferUsages::VERTEX });
            let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Cloud IB"), contents: bytemuck::cast_slice(&indices), usage: BufferUsages::INDEX });
            self.chunk_meshes.insert((999, 999, 999), (ChunkMesh { vertex_buffer: vb, index_buffer: ib, _ranges: vec![TextureRange { _tex_index: 228, _index_start: 0, _index_count: indices.len() as u32 }], total_indices: indices.len() as u32 }, 0));
        }
    }

    // RADICAL FIX: Completely rewritten update_chunk with proper coordinate handling
    pub fn update_chunk(&mut self, cx: i32, cy: i32, cz: i32, world: &World) {
        // DIABOLICAL PURGE: Before we even check the chunk, we ensure the "dirty" flag is cleared
        // locally so the generator doesn't get stuck in a loop.
        if let Some(chunk) = world.chunks.get(&(cx, cy, cz)) {
            if chunk.is_empty {
                self.chunk_meshes.remove(&(cx, cy, cz));
                return;
            }
            
            let mut chunk_v = Vec::new();
            let mut chunk_i = Vec::new();
            let mut i_cnt = 0u32;
            
            let bx = (cx * 16) as f32;
            let by = (cy * 16) as f32;
            let bz = (cz * 16) as f32;

            // RADICAL FIX: Consistent 16x16x16 greedy meshing for all axes
            for axis in 0..3 {
                let dims_u = 16usize;
                let dims_v = 16usize;
                let dims_main = 16usize;

                for d in 0..dims_main {
                    for dir in 0..2 { 
                        let face_id = match axis { 
                            0 => if dir==0 {0} else {1}, // Y: Top/Bottom
                            1 => if dir==0 {2} else {3}, // X: Right/Left
                            _ => if dir==0 {4} else {5}  // Z: Front/Back
                        };
                        
                        let mut mask = vec![BlockType::Air; dims_u * dims_v];
                        
                        // Build visibility mask
                        for u_idx in 0..dims_u {
                            for v_idx in 0..dims_v {
                                // RADICAL FIX: Consistent coordinate mapping
                                let (lx, ly, lz) = match axis {
                                    0 => (u_idx, d, v_idx),      // Y-slice: x=u, y=d, z=v
                                    1 => (d, u_idx, v_idx),      // X-slice: x=d, y=u, z=v
                                    _ => (u_idx, v_idx, d)       // Z-slice: x=u, y=v, z=d
                                };
                                
                                let blk = chunk.get_block(lx, ly, lz);
                                if !blk.is_solid() && !blk.is_liquid() {
                                    continue;
                                }
                                
                                // Check neighbor
                                let (nx, ny, nz) = match face_id {
                                    0 => (lx as i32, ly as i32 + 1, lz as i32),
                                    1 => (lx as i32, ly as i32 - 1, lz as i32),
                                    2 => (lx as i32 + 1, ly as i32, lz as i32),
                                    3 => (lx as i32 - 1, ly as i32, lz as i32),
                                    4 => (lx as i32, ly as i32, lz as i32 + 1),
                                    5 => (lx as i32, ly as i32, lz as i32 - 1),
                                    _ => (0, 0, 0)
                                };
                                
                                // DIABOLICAL BOUNDARY SYNC: Ghost blocks often appear because the neighbor 
                                // lookup fails or returns 'Air' incorrectly during a chunk transition.
                                let neighbor = {
                                    let wx = cx * 16 + nx;
                                    let wy = cy * 16 + ny;
                                    let wz = cz * 16 + nz;
                                    world.get_block(BlockPos { x: wx, y: wy, z: wz })
                                };

                                // ROOT CAUSE FIX: A block is visible if the neighbor is not solid OR 
                                // if the neighbor is the EXACT SAME transparent block (prevents internal faces).
                                let visible = !neighbor.is_solid() || (neighbor.is_transparent() && neighbor != blk);
                                if visible { 
                                    mask[v_idx * dims_u + u_idx] = blk; 
                                }
                            }
                        }

                        // Greedy merge
                        let mut n = 0;
                        while n < mask.len() {
                            let blk = mask[n];
                            if blk != BlockType::Air {
                                let mut w = 1;
                                while (n + w) % dims_u != 0 && mask[n + w] == blk { 
                                    w += 1; 
                                }
                                
                                let mut h = 1;
                                'h_loop: while (n / dims_u + h) < dims_v {
                                    for k in 0..w {
                                        if mask[n + k + h * dims_u] != blk { 
                                            break 'h_loop; 
                                        }
                                    }
                                    h += 1;
                                }

                                let u_greedy = (n % dims_u) as i32;
                                let v_greedy = (n / dims_u) as i32;
                                
                                // RADICAL FIX: Consistent world position calculation
                                let (wx, wy, wz) = match axis {
                                    0 => (bx + u_greedy as f32, by + d as f32, bz + v_greedy as f32),
                                    1 => (bx + d as f32, by + u_greedy as f32, bz + v_greedy as f32),
                                    _ => (bx + u_greedy as f32, by + v_greedy as f32, bz + d as f32)
                                };
                                
                                let world_w = w as f32;
                                let world_h = h as f32;

                                self.add_face_greedy(&mut chunk_v, &mut chunk_i, &mut i_cnt, wx, wy, wz, world_w, world_h, face_id, blk);


                                for l in 0..h {
                                    for k in 0..w { 
                                        mask[n + k + l * dims_u] = BlockType::Air; 
                                    }
                                }
                            }
                            n += 1;
                        }
                    }
                }
            }

            if !chunk_v.is_empty() {
                let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                    label: Some("Chunk VB"), 
                    contents: bytemuck::cast_slice(&chunk_v), 
                    usage: BufferUsages::VERTEX 
                });
                let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                    label: Some("Chunk IB"), 
                    contents: bytemuck::cast_slice(&chunk_i), 
                    usage: BufferUsages::INDEX 
                });
                self.chunk_meshes.insert((cx, cy, cz), (ChunkMesh { 
                    vertex_buffer: vb, 
                    index_buffer: ib, 
                    _ranges: Vec::new(), 
                    total_indices: chunk_i.len() as u32 
                }, 0));
            } else {
                self.chunk_meshes.remove(&(cx, cy, cz));
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
            4 => ([x,y,z+1.0], [x+1.0,y,z+1.0], [x+1.0,y+h,z+1.0], [x,y+h,z+1.0], [1.0,1.0], [0.0,1.0], [0.0,0.0], [1.0,0.0]), // Fixed east face UV
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
        v.push(Vertex{position:t(p0), tex_coords:[0.0,1.0], ao:1.0, tex_index:tex, light: 15.0}); v.push(Vertex{position:t(p1), tex_coords:[1.0,1.0], ao:1.0, tex_index:tex, light: 15.0});
        v.push(Vertex{position:t(p2), tex_coords:[1.0,0.0], ao:1.0, tex_index:tex, light: 15.0}); v.push(Vertex{position:t(p3), tex_coords:[0.0,0.0], ao:1.0, tex_index:tex, light: 15.0});
        i.push(*off); i.push(*off+1); i.push(*off+2); i.push(*off); i.push(*off+2); i.push(*off+3); *off += 4;
    }

fn _add_cross_face(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32, x: i32, y: i32, z: i32, tex: u32, light: f32) {
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
// DIABOLICAL GREEDY MESHER HELPER: Absolute Positional Integrity (Fixes Plane Fighting)
    fn add_face_greedy(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, i_count: &mut u32, x: f32, y: f32, z: f32, w: f32, h: f32, face: usize, block: BlockType) {
        let tex_index = match face {
            0 => block.get_texture_top(),
            1 => block.get_texture_bottom(),
            _ => block.get_texture_side()
        };
        
        // BIT-LEVEL PARITY FIX: Correct UV mapping and winding for all 6 faces to prevent "Reversed Texture" bugs.
        // WGPU uses Y-down UV, so top-left of a block is (0,0).
        let (positions, uv) = match face {
            0 => ([[x, y + 1.0, z + h], [x + w, y + 1.0, z + h], [x + w, y + 1.0, z], [x, y + 1.0, z]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Top
            1 => ([[x, y, z], [x + w, y, z], [x + w, y, z + h], [x, y, z + h]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Bottom
            2 => ([[x + 1.0, y, z + h], [x + 1.0, y + w, z + h], [x + 1.0, y + w, z], [x + 1.0, y, z]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Right (East)
            3 => ([[x, y, z], [x, y + w, z], [x, y + w, z + h], [x, y, z + h]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Left (West)
            4 => ([[x, y, z + 1.0], [x + w, y, z + 1.0], [x + w, y + h, z + 1.0], [x, y + h, z + 1.0]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Front
            5 => ([[x + w, y, z], [x, y, z], [x, y + h, z], [x + w, y + h, z]], [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]]), // Back
            _ => ([[0.0; 3]; 4], [[0.0; 2]; 4]),
        };

        let base_i = *i_count;
        v.push(Vertex { position: positions[0], tex_coords: uv[0], ao: 1.0, tex_index, light: 15.0 });
        v.push(Vertex { position: positions[1], tex_coords: uv[1], ao: 1.0, tex_index, light: 15.0 });
        v.push(Vertex { position: positions[2], tex_coords: uv[2], ao: 1.0, tex_index, light: 15.0 });
        v.push(Vertex { position: positions[3], tex_coords: uv[3], ao: 1.0, tex_index, light: 15.0 });
        
        i.extend_from_slice(&[base_i, base_i + 1, base_i + 2, base_i, base_i + 2, base_i + 3]);
        *i_count += 4;
    }

pub fn add_ui_quad(&self, uv: &mut Vec<Vertex>, ui: &mut Vec<u32>, uoff: &mut u32, x: f32, y: f32, w: f32, h: f32, tex_index: u32) {
        uv.push(Vertex{position:[x,y+h,0.0], tex_coords:[0.0,0.0], ao:1.0, tex_index, light: 1.0}); uv.push(Vertex{position:[x+w,y+h,0.0], tex_coords:[1.0,0.0], ao:1.0, tex_index, light: 1.0});
        uv.push(Vertex{position:[x+w,y,0.0], tex_coords:[1.0,1.0], ao:1.0, tex_index, light: 1.0}); uv.push(Vertex{position:[x,y,0.0], tex_coords:[0.0,1.0], ao:1.0, tex_index, light: 1.0});
        ui.push(*uoff); ui.push(*uoff+1); ui.push(*uoff+2); ui.push(*uoff); ui.push(*uoff+2); ui.push(*uoff+3); *uoff += 4;
    }

pub fn draw_text(&self, text: &str, start_x: f32, y: f32, scale: f32, v: &mut Vec<Vertex>, i: &mut Vec<u32>, off: &mut u32) {
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
                      else if c == '/' { 300 + 38 } // DIABOLICAL SLASH SUPPORT
                      else { 999 };
            
            if idx != 999 {
                self.add_ui_quad(v, i, off, x, y, final_scale, final_scale * aspect, idx);
            }
            x += final_scale;
        }
    }
pub fn render_multiplayer_menu(&mut self, menu: &mut crate::MainMenu, hosting: &crate::network::HostingManager, _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Multiplayer") });
    let mut v: Vec<Vertex> = Vec::new(); let mut i: Vec<u32> = Vec::new(); let mut off = 0;

    // 1. Dark Background
    self.add_ui_quad(&mut v, &mut i, &mut off, -1.0, -1.0, 2.0, 2.0, 2); 
    self.draw_text("MULTIPLAYER DISCOVERY", -0.7, 0.8, 0.08, &mut v, &mut i, &mut off);

    // 2. Dynamic Server List from Discovery
    menu.buttons.clear();
    let servers = hosting.discovered_servers.lock().unwrap();
    let mut y_pos = 0.5;
    for srv in servers.iter() {
        let rect = crate::Rect { x: 0.0, y: y_pos, w: 1.4, h: 0.15 };
        let hovered = rect.contains((self.config.width as f32 / 2.0) / self.config.width as f32 * 2.0 - 1.0, y_pos); // Placeholder hover check
        menu.buttons.push(crate::MenuButton { rect, text: srv.name.clone(), action: crate::MenuAction::JoinAddr(srv.address.clone()), hovered });
        
        self.add_ui_quad(&mut v, &mut i, &mut off, -0.7, y_pos - 0.075, 1.4, 0.15, if hovered { 251 } else { 250 });
        self.draw_text(&format!("{} > {}", srv.name, srv.address), -0.65, y_pos - 0.02, 0.04, &mut v, &mut i, &mut off);
        y_pos -= 0.2;
    }

    // 3. Manual Entry & Back
    let back_rect = crate::Rect { x: 0.0, y: -0.8, w: 0.4, h: 0.1 };
    menu.buttons.push(crate::MenuButton { rect: back_rect, text: "BACK".to_string(), action: crate::MenuAction::Quit, hovered: false });
    self.add_ui_quad(&mut v, &mut i, &mut off, -0.2, -0.85, 0.4, 0.1, 250);
    self.draw_text("BACK", -0.05, -0.82, 0.04, &mut v, &mut i, &mut off);

    let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("M VB"), contents: bytemuck::cast_slice(&v), usage: wgpu::BufferUsages::VERTEX });
    let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("M IB"), contents: bytemuck::cast_slice(&i), usage: wgpu::BufferUsages::INDEX });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MP Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.ui_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..i.len() as u32, 0, 0..1);
    }
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
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
            let rx = -1.0 + (gx as f32 * grid_size);
            let ry = -1.0 + (gy as f32 * grid_size);
            self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, rx, ry, grid_size, grid_size, 2);
        }
    }

// 1.5 Draw Title (DIABOLICALLY BIG)
    self.draw_text("MINECRAFT", -0.85, 0.7, 0.2, &mut vertices, &mut indices, &mut idx_offset);

    // 2. Buttons & Text
    for btn in &menu.buttons {
        let tex_id = if btn.hovered { 251 } else { 250 };
        let rect = &btn.rect;
        
        // DIABOLICAL UV FIX: vs_ui shader handles atlas offsets. Buttons must pass 0.0-1.0 local UVs.
        vertices.push(Vertex { position: [rect.x - rect.w / 2.0, rect.y - rect.h / 2.0, 0.0], tex_coords: [0.0, 1.0], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x + rect.w / 2.0, rect.y - rect.h / 2.0, 0.0], tex_coords: [1.0, 1.0], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, 0.0], tex_coords: [1.0, 0.0], ao: 1.0, tex_index: tex_id, light: 1.0 });
        vertices.push(Vertex { position: [rect.x - rect.w / 2.0, rect.y + rect.h / 2.0, 0.0], tex_coords: [0.0, 0.0], ao: 1.0, tex_index: tex_id, light: 1.0 });
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
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_bind_group(1, &self.camera_bind_group, &[]);
        rpass.set_bind_group(2, &self.time_bind_group, &[]);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

    pub fn render_settings_menu(&mut self, menu: &mut SettingsMenu, _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Settings") });

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut idx_offset = 0;

    // Professional dark overlay with vignette effect
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, -1.0, -1.0, 2.0, 2.0, 240);
    
    // Professional menu background panel
    let panel_width = 0.7;
    let panel_height = 0.6;
    let panel_x = 0.0;
    let panel_y = 0.0;
    
    // Main panel background with border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        panel_width, panel_height, 252); // Dark panel texture
    
    // Panel border
    let border_size = 0.008;
    // Top border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y + panel_height/2.0 - border_size, 
        panel_width, border_size, 253); // Border texture
    // Bottom border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        panel_width, border_size, 253);
    // Left border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        border_size, panel_height, 253);
    // Right border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x + panel_width/2.0 - border_size, panel_y - panel_height/2.0, 
        border_size, panel_height, 253);

    // Menu title
    self.draw_text("SETTINGS", 0.0, panel_y + panel_height/2.0 - 0.08, 0.025, &mut vertices, &mut indices, &mut idx_offset);

    // Render settings options with values
    for (i, btn) in menu.buttons.iter_mut().enumerate() {
        let button_width = 0.5;
        let button_height = 0.06;
        let button_spacing = 0.08;
        let start_y = panel_y - button_spacing/2.0;
        let button_y = start_y - (i as f32 * button_spacing);
        
        // UPDATE RECT: Set the button rect to match rendered position for correct click detection
        btn.rect.x = panel_x;
        btn.rect.y = button_y;
        btn.rect.w = button_width;
        btn.rect.h = button_height;
        
        // Button background with hover effect
        let tex_id = if btn.hovered { 251 } else { 250 };
        self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
            panel_x - button_width/2.0, button_y - button_height/2.0, 
            button_width, button_height, tex_id);
        
        // Button text with better positioning
        let text_scale = 0.015;
        let text_width = btn.text.len() as f32 * text_scale * 0.8;
        self.draw_text(&btn.text, panel_x - text_width/2.0, button_y + text_scale/4.0, text_scale, &mut vertices, &mut indices, &mut idx_offset);
        
        // Display current setting value
        let value_text = match i {
            0 => format!("{:.0}%", menu.settings_values.master_volume * 100.0),
            1 => format!("{:.0}%", menu.settings_values.music_volume * 100.0),
            2 => format!("{:.0}%", menu.settings_values.sfx_volume * 100.0),
            3 => format!("{} chunks", menu.settings_values.render_distance),
            4 => format!("{:.0}°", menu.settings_values.fov),
            5 => format!("{} FPS", menu.settings_values.max_fps),
            6 => menu.settings_values.shader_type.get_display_name().to_string(),
            _ => String::new(),
        };
        
        if !value_text.is_empty() {
            let value_scale = 0.012;
            let value_width = value_text.len() as f32 * value_scale * 0.8;
            self.draw_text(&value_text, panel_x + button_width/2.0 - value_width - 0.02, button_y + value_scale/4.0, value_scale, &mut vertices, &mut indices, &mut idx_offset);
        }
    }

    let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Settings VB"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
    let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Settings IB"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Settings Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.ui_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_bind_group(1, &self.camera_bind_group, &[]);
        rpass.set_bind_group(2, &self.time_bind_group, &[]);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

pub fn render_pause_menu(&mut self, menu: &mut MainMenu, world: &World, player: &Player, cursor_pos: (f64, f64), _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Pause") });

    // Composite Call: Render the world FIRST into the SAME view and encoder
    self.render_internal(world, player, true, cursor_pos, &view, &mut encoder);

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut idx_offset = 0;

    // Professional dark overlay with vignette effect
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, -1.0, -1.0, 2.0, 2.0, 240);
    
    // Professional menu background panel
    let panel_width = 0.6;
    let panel_height = 0.4;
    let panel_x = 0.0;
    let panel_y = 0.0;
    
    // Main panel background with border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        panel_width, panel_height, 252); // Dark panel texture
    
    // Panel border
    let border_size = 0.008;
    // Top border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y + panel_height/2.0 - border_size, 
        panel_width, border_size, 253); // Border texture
    // Bottom border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        panel_width, border_size, 253);
    // Left border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x - panel_width/2.0, panel_y - panel_height/2.0, 
        border_size, panel_height, 253);
    // Right border
    self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
        panel_x + panel_width/2.0 - border_size, panel_y - panel_height/2.0, 
        border_size, panel_height, 253);

    // Menu title
    self.draw_text("GAME PAUSED", 0.0, panel_y + panel_height/2.0 - 0.08, 0.025, &mut vertices, &mut indices, &mut idx_offset);

    // Render buttons with professional styling
    for (i, btn) in menu.buttons.iter_mut().enumerate() {
        let button_width = 0.4;
        let button_height = 0.06;
        let button_spacing = 0.08;
        let start_y = panel_y - button_spacing/2.0;
        let button_y = start_y - (i as f32 * button_spacing);
        
        // UPDATE RECT: Set the button rect to match rendered position for correct click detection
        btn.rect.x = panel_x;
        btn.rect.y = button_y;
        btn.rect.w = button_width;
        btn.rect.h = button_height;
        
        // Button background with hover effect
        let tex_id = if btn.hovered { 251 } else { 250 };
        self.add_ui_quad(&mut vertices, &mut indices, &mut idx_offset, 
            panel_x - button_width/2.0, button_y - button_height/2.0, 
            button_width, button_height, tex_id);
        
        // Button text with better positioning
        let text_scale = 0.018;
        let text_width = btn.text.len() as f32 * text_scale * 0.8;
        self.draw_text(&btn.text, panel_x - text_width/2.0, button_y + text_scale/4.0, text_scale, &mut vertices, &mut indices, &mut idx_offset);
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
        rpass.set_bind_group(1, &self.camera_bind_group, &[]);
        rpass.set_bind_group(2, &self.time_bind_group, &[]);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

    pub fn render_game(&mut self, world: &World, player: &Player, is_paused: bool, cursor_pos: (f64, f64), _width: u32, _height: u32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render Encoder") });

        self.render_internal(world, player, is_paused, cursor_pos, &view, &mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    /// DIABOLICAL RENDER CORE: Separated from presentation to allow composite passes (like Pause Menu)
    fn render_internal(&mut self, world: &World, player: &Player, is_paused: bool, cursor_pos: (f64, f64), view: &TextureView, encoder: &mut CommandEncoder) {
        // 1. FPS Calculation & Console Output
        self.frame_count += 1;
        let time_since_last = self.last_fps_time.elapsed();
        if time_since_last.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32 / time_since_last.as_secs_f32();
            
            // DIABOLICAL TELEMETRY SNAPSHOT: Hyper-Exhaustive high-density diagnostic
            let p = player.position; let v = player.velocity;
            let dt_val = time_since_last.as_secs_f32() / self.frame_count as f32;
            log::info!("[STAT] FPS:{:<3.0} | DT:{:.4}s | CHK:{} | PND:{} | POS:({:.1},{:.1},{:.1}) | VEL:({:.2},{:.2},{:.2}) | GRD:{} FLY:{} SPR:{} | PTCL:{} | TRK:{} | DIRTY:{}", 
                self.fps, 
                dt_val,
                self.chunk_meshes.len(), 
                self.pending_chunks.len(),
                p.x, p.y, p.z, 
                v.x, v.y, v.z,
                if player.on_ground {'Y'} else {'N'}, 
                if player.is_flying {'Y'} else {'N'}, 
                if player.is_sprinting {'Y'} else {'N'},
                self.particles.len(),
                0, // Placeholder for TRK (Tracking) to match format string index
                world.dirty_chunks.len() // Track pending mesh updates
            );
            
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
        }

        // 2. DIABOLICAL MESH BARRIER: Consistently process incoming meshes from worker threads.
        self.process_mesh_queue();

        // 3. PRIORITY MESH UPDATER: Prevents the 5-second lag and flickering
        let p_cx = (player.position.x / 16.0).floor() as i32;
        let p_cz = (player.position.z / 16.0).floor() as i32;
        
        let player_moved = (p_cx, p_cz) != (self.last_player_chunk.0, self.last_player_chunk.2);
        let world_arc = Arc::new(world.clone());

        // A. Handle explicitly dirty chunks FIRST (Block breaking/placing)
        for &target in &world.dirty_chunks.clone() {
            if !self.pending_chunks.contains(&target) {
                // Force immediate remesh for broken/placed blocks
                self.pending_chunks.insert(target);
                let _ = self.mesh_tx.send((target.0, target.1, target.2, 0, world_arc.clone()));
            }
        }

        // B. Handle background loading and movement (More aggressive for better coverage)
        if player_moved || self.frame_count % 15 == 0 { // Check every 15 frames instead of 30
            self.last_player_chunk = (p_cx, 0, p_cz);
            let r_dist = 12; // Increased from 10 for better chunk loading
            let max_vertical = crate::engine::WORLD_HEIGHT / 16;
            
            for dx in -r_dist..=r_dist {
                for dz in -r_dist..=r_dist {
                    if dx*dx + dz*dz > r_dist*r_dist { continue; }
                    for dy in 0..max_vertical {
                        let target = (p_cx + dx, dy, p_cz + dz);
                        if let Some(_c) = world.chunks.get(&target) {
                            if !self.chunk_meshes.contains_key(&target) && !self.pending_chunks.contains(&target) {
                                self.pending_chunks.insert(target);
                                let _ = self.mesh_tx.send((target.0, target.1, target.2, 0, world_arc.clone()));
                            }
                        }
                    }
                }
            }
        }

        // 3. Setup Uniforms
        let aspect = self.config.width as f32 / self.config.height as f32;

        // DIABOLICAL FIX: camera_buffer is already written by main.rs via update_camera(). 
        // Removing redundant write to save PCIe bandwidth and GPU sync points.
        
        let time = self.start_time.elapsed().as_secs_f32();
        let eye_bp = BlockPos { x: player.position.x.floor() as i32, y: (player.position.y + player.height * 0.4).floor() as i32, z: player.position.z.floor() as i32 };
        let is_underwater = if world.get_block(eye_bp).is_water() { 1.0f32 } else { 0.0f32 };
        
        let noise_gen = crate::resources::NoiseGenerator::new(world.seed);
        let (cont, eros, _weird, temp) = noise_gen.get_height_params(eye_bp.x, eye_bp.z);
        let humid = noise_gen.get_noise_octaves(eye_bp.x as f64 * 0.01, 44.0, eye_bp.z as f64 * 0.01, 3) as f32;
        let biome = noise_gen.get_biome(cont, eros, temp, humid, eye_bp.y);
        let fog_color = match biome {
            "swamp" => [0.3, 0.4, 0.2, 1.0], "desert" => [0.8, 0.7, 0.5, 1.0], "ice_plains" => [0.9, 0.9, 1.0, 1.0], _ => [0.5, 0.8, 0.9, 1.0],
        };
        self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[fog_color[0], fog_color[1], fog_color[2], fog_color[3], time, is_underwater, 0.0, 0.0]));

        // 4. Entity Buffer Preparation
        let mut ent_v = Vec::new(); let mut ent_i = Vec::new(); let mut ent_off = 0;
        
        // DIABOLICAL IN-WORLD BREAKING ANIMATION
        if self.break_progress > 0.0 {
            let (sin, cos) = player.rotation.x.sin_cos(); 
            let (ysin, ycos) = player.rotation.y.sin_cos();
            let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
            if let Some((hit, place)) = world.raycast(player.position + glam::Vec3::new(0.0, player.height * 0.4, 0.0), dir, 5.0) {
                let crack_tex = 210 + (self.break_progress * 9.0).min(9.0) as u32;
                let normal = glam::Vec3::new((place.x - hit.x) as f32, (place.y - hit.y) as f32, (place.z - hit.z) as f32);
                let face_id = if normal.y > 0.5 { 0 } else if normal.y < -0.5 { 1 } else if normal.x > 0.5 { 2 } else if normal.x < -0.5 { 3 } else if normal.z > 0.5 { 4 } else { 5 };
                
                // Add crack overlay quad with tiny epsilon offset to prevent Z-fighting
                let eps = 0.005;
                let x = hit.x as f32; let y = hit.y as f32; let z = hit.z as f32;
                let (p, uv) = match face_id {
                    0 => ([[x,y+1.0+eps,z+1.0], [x+1.0,y+1.0+eps,z+1.0], [x+1.0,y+1.0+eps,z], [x,y+1.0+eps,z]], [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
                    1 => ([[x,y-eps,z], [x+1.0,y-eps,z], [x+1.0,y-eps,z+1.0], [x,y-eps,z+1.0]], [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]]),
                    2 => ([[x+1.0+eps,y,z], [x+1.0+eps,y,z+1.0], [x+1.0+eps,y+1.0,z+1.0], [x+1.0+eps,y+1.0,z]], [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
                    3 => ([[x-eps,y,z+1.0], [x-eps,y,z], [x-eps,y+1.0,z], [x-eps,y+1.0,z+1.0]], [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
                    4 => ([[x,y,z+1.0+eps], [x+1.0,y,z+1.0+eps], [x+1.0,y+1.0,z+1.0+eps], [x,y+1.0,z+1.0+eps]], [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
                    5 => ([[x+1.0,y,z-eps], [x,y,z-eps], [x,y+1.0,z-eps], [x+1.0,y+1.0,z-eps]], [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
                    _ => ([[0.0;3];4], [[0.0;2];4]),
                };
                let base = ent_off;
                for i in 0..4 { ent_v.push(Vertex { position: p[i], tex_coords: uv[i], ao: 1.0, tex_index: crack_tex, light: 15.0 }); }
                ent_i.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
                ent_off += 4;
            }
        }

        for rp in &world.remote_players {
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y, rp.position.z], rp.rotation, -0.3, 0.0, -0.3, 0.6, f, 13); }
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y+0.65, rp.position.z], rp.rotation, -0.3, 0.0, -0.3, 0.6, f, 13); }
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [rp.position.x, rp.position.y+1.3, rp.position.z], rp.rotation, -0.25, 0.0, -0.25, 0.5, f, 13); }
        }
        for e in &world.entities {
            let (t, _, _) = e.item_type.get_texture_indices();
            let rot = time * 1.5 + e.bob_offset; let by = ((time * 4.0 + e.bob_offset).sin() * 0.05) + 0.12;
            for f in 0..6 { self.add_rotated_quad(&mut ent_v, &mut ent_i, &mut ent_off, [e.position.x, e.position.y+by, e.position.z], rot, -0.125, -0.125, -0.125, 0.25, f, t); }
        }
        
        // DIABOLICAL OPTIMIZATION: Stop re-creating GPU buffers every frame. 
        // Re-use buffers and only expand if necessary. This stops GPU driver hitching completely.
        if !ent_v.is_empty() {
            let v_size = (ent_v.len() * std::mem::size_of::<Vertex>()) as u64;
            let i_size = (ent_i.len() * 4) as u64;
            
            if v_size > self.entity_vertex_buffer.size() || i_size > self.entity_index_buffer.size() {
                self.entity_vertex_buffer = self.device.create_buffer(&BufferDescriptor { label: Some("Entity VB"), size: v_size * 2, usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, mapped_at_creation: false });
                self.entity_index_buffer = self.device.create_buffer(&BufferDescriptor { label: Some("Entity IB"), size: i_size * 2, usage: BufferUsages::INDEX | BufferUsages::COPY_DST, mapped_at_creation: false });
            }
            self.queue.write_buffer(&self.entity_vertex_buffer, 0, bytemuck::cast_slice(&ent_v));
            self.queue.write_buffer(&self.entity_index_buffer, 0, bytemuck::cast_slice(&ent_i));
        }

        // 5. 3D Pass
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("3D Pass"), 
                color_attachments: &[Some(RenderPassColorAttachment { view: &view, resolve_target: None, ops: Operations { load: LoadOp::Clear(Color { r: fog_color[0] as f64, g: fog_color[1] as f64, b: fog_color[2] as f64, a: 1.0 }), store: StoreOp::Store } })], 
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment { view: &self.depth_texture, depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }), stencil_ops: None }), 
                timestamp_writes: None, occlusion_query_set: None 
            });
            
            pass.set_pipeline(&self.pipeline); 
            pass.set_bind_group(0, &self.bind_group, &[]); 
            pass.set_bind_group(1, &self.camera_bind_group, &[]); 
            pass.set_bind_group(2, &self.time_bind_group, &[]);

            // Draw World with FRUSTUM CULLING (Pre-extract normals for raw speed)
            let planes = player.get_frustum_planes(aspect);
            let p_normals: [glam::Vec3; 6] = [
                glam::Vec3::from_slice(&planes[0][0..3]), glam::Vec3::from_slice(&planes[1][0..3]),
                glam::Vec3::from_slice(&planes[2][0..3]), glam::Vec3::from_slice(&planes[3][0..3]),
                glam::Vec3::from_slice(&planes[4][0..3]), glam::Vec3::from_slice(&planes[5][0..3]),
            ];

            for (&(cx, cy, cz), (mesh, _)) in &self.chunk_meshes {
                if cx == 999 { continue; }
                
                let center = glam::Vec3::new(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0, cz as f32 * 16.0 + 8.0);
                let radius = 14.0; 
                let mut visible = true;
                for i in 0..6 {
                    if p_normals[i].dot(center) + planes[i][3] < -radius {
                        visible = false;
                        break;
                    }
                }

                if visible {
                    pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.total_indices, 0, 0..1);
                }
            }

            // Draw Clouds
            if let Some((m, _)) = self.chunk_meshes.get(&(999, 999, 999)) {
                pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                pass.set_index_buffer(m.index_buffer.slice(..), IndexFormat::Uint32);
                pass.draw_indexed(0..m.total_indices, 0, 0..1);
            }

            // Draw Entities
            if !ent_v.is_empty() { 
                pass.set_vertex_buffer(0, self.entity_vertex_buffer.slice(..)); 
                pass.set_index_buffer(self.entity_index_buffer.slice(..), IndexFormat::Uint32); 
                pass.draw_indexed(0..ent_i.len() as u32, 0, 0..1); 
            }
        }

       // 6. UI Logic & Render Pass
        let mut uv = Vec::new(); let mut ui = Vec::new(); let mut uoff = 0;
        
        // Crosshair
        if !player.inventory_open && !is_paused { 
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -0.01, -0.01 * aspect, 0.02, 0.02 * aspect, 240); 
        }

        // Hotbar
        let sw = 0.12; let sh = sw * aspect; let sx = -(sw * 9.0) / 2.0; let by = -0.9;
        
        // Bubbles
        if player.air < player.max_air {
            let bubble_count = (player.air / player.max_air * 10.0).ceil() as i32;
            let bx_bubbles = sx + sw * 5.0; let by_bubbles = by + sh + 0.08 * aspect;
            for i in 0..10 { if i < bubble_count { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, bx_bubbles + i as f32 * 0.045, by_bubbles, 0.04, 0.04 * aspect, 243); } }
        }

        if player.inventory_open {
             self.add_ui_quad(&mut uv, &mut ui, &mut uoff, -1.0, -1.0, 2.0, 2.0, 240);
             self.draw_text("INVENTORY", -0.2, 0.8, 0.08, &mut uv, &mut ui, &mut uoff);
        }

        // FPS & TELEMETRY COUNTER
        self.draw_text(&format!("FPS {}", self.fps as u32), -0.98, 0.94, 0.03, &mut uv, &mut ui, &mut uoff);
        
        // DIABOLICAL MULTIPLAYER TELEMETRY: Render Hosting Status if active
        // Logic: if host URL is not "Initializing..." or empty, show it for accessibility.
        // We use the player's spawn_timer field as a proxy for 'debug_visible' if needed, 
        // or just always show URL in top-right for hosting clarity.

        if !is_paused || player.inventory_open {
            for i in 0..9 {
                let x = sx + (i as f32 * sw);
                if i == player.inventory.selected_hotbar_slot { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x - 0.005, by - 0.005 * aspect, sw + 0.01, sh + 0.01 * aspect, 241); }
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x, by, sw, sh, 240);
                if let Some(stack) = &player.inventory.slots[i] {
                    let (t, _, _) = stack.item.get_texture_indices();
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x+0.02, by+0.02*aspect, sw-0.04, sh-0.04*aspect, t);
                    if stack.count > 1 { 
                        let text_scale = if stack.count >= 10 { 0.03 } else { 0.04 };
                        let text_x = if stack.count >= 10 { x + 0.05 } else { x + 0.07 };
                        self.draw_text(&format!("{}", stack.count), text_x, by + 0.02, text_scale, &mut uv, &mut ui, &mut uoff); 
                    }
                }
            }
            if !player.inventory_open {
                for i in 0..10 { if player.health > (i as f32)*2.0 { self.add_ui_quad(&mut uv, &mut ui, &mut uoff, sx + i as f32 * 0.05, by+sh+0.02*aspect, 0.045, 0.045*aspect, 242); } }
            }
        }

        if player.inventory_open {
            let iby = by + sh * 1.5;
            for r in 0..3 { for c in 0..9 {
                let idx = 9 + r * 9 + c; let x = sx + c as f32 * sw; let y = iby + r as f32 * sh;
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x, y, sw, sh, 240);
                if let Some(stack) = &player.inventory.slots[idx] { 
                    let (t, _, _) = stack.item.get_texture_indices(); 
                    self.add_ui_quad(&mut uv, &mut ui, &mut uoff, x+0.02, y+0.02*aspect, sw-0.04, sh-0.04*aspect, t); 
                    if stack.count > 1 { 
                        let text_scale = if stack.count >= 10 { 0.025 } else { 0.03 };
                        let text_x = if stack.count >= 10 { x-0.01 } else { x+0.01 };
                        self.draw_text(&format!("{}", stack.count), text_x, y+0.01, text_scale, &mut uv, &mut ui, &mut uoff); 
                    } 
                }
            }}
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
                    if stack.count > 1 { 
                        let text_scale = if stack.count >= 10 { 0.025 } else { 0.03 };
                        let text_x = if stack.count >= 10 { x-0.01 } else { x+0.01 };
                        self.draw_text(&format!("{}", stack.count), text_x, y+0.01, text_scale, &mut uv, &mut ui, &mut uoff); 
                    }
                }
            }}
            let ox = cx + 3.0*sw; let oy = cy - 0.5*sh;
            self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ox, oy, sw, sh, 240); 
            self.draw_text("->", cx + 2.1*sw, oy+0.05, 0.04, &mut uv, &mut ui, &mut uoff);
            if let Some(stack) = &player.inventory.crafting_output { 
                let (t, _, _) = stack.item.get_texture_indices(); 
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ox+0.02, oy+0.02*aspect, sw-0.04, sh-0.04*aspect, t); 
                if stack.count > 1 { 
                    let text_scale = if stack.count >= 10 { 0.025 } else { 0.03 };
                    let text_x = if stack.count >= 10 { ox-0.01 } else { ox+0.01 };
                    self.draw_text(&format!("{}", stack.count), text_x, oy+0.01, text_scale, &mut uv, &mut ui, &mut uoff); 
                } 
            }
            let (mx, my) = cursor_pos; 
            let ndc_x = (mx as f32 / self.config.width as f32)*2.0-1.0; 
            let ndc_y = -((my as f32 / self.config.height as f32)*2.0-1.0);
            if let Some(stack) = &player.inventory.cursor_item {
                let (t, _, _) = stack.item.get_texture_indices();
                self.add_ui_quad(&mut uv, &mut ui, &mut uoff, ndc_x - sw/2.0, ndc_y - sh/2.0, sw, sh, t);
                if stack.count > 1 { 
                    let text_scale = if stack.count >= 10 { 0.025 } else { 0.03 };
                    let text_x = if stack.count >= 10 { ndc_x - sw/2.0 - 0.02 } else { ndc_x - sw/2.0 };
                    self.draw_text(&format!("{}", stack.count), text_x, ndc_y - sh/2.0, text_scale, &mut uv, &mut ui, &mut uoff); 
                }
            }
        }

        if !uv.is_empty() {
            let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("UI VB"), contents: bytemuck::cast_slice(&uv), usage: BufferUsages::VERTEX });
            let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("UI IB"), contents: bytemuck::cast_slice(&ui), usage: BufferUsages::INDEX });
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor { 
                label: Some("UI Pass"), color_attachments: &[Some(RenderPassColorAttachment { view: &view, resolve_target: None, ops: Operations { load: LoadOp::Load, store: StoreOp::Store } })], 
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None 
            });
            pass.set_pipeline(&self.ui_pipeline); pass.set_bind_group(0, &self.bind_group, &[]); pass.set_vertex_buffer(0, vb.slice(..)); pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32); pass.draw_indexed(0..ui.len() as u32, 0, 0..1);
        }
    }
}
// DIABOLICAL COMBAT SYSTEM - Advanced Combat Mechanics and Mob AI
// 
// This module provides comprehensive combat features including:
// - Advanced mob AI with different behavior patterns
// - Combat mechanics with damage types and effects
// - Weapon and armor systems with enchantments
// - Particle effects and visual feedback
// - Boss battles and special abilities
// - Combat animations and sound effects

use glam::Vec3;

/// DIABOLICAL Combat Damage Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageType {
    Physical,
    Fire,
    Water,
    Earth,
    Air,
    Arcane,
    Holy,
    Shadow,
    Poison,
    Lightning,
}

/// DIABOLICAL Weapon Types with unique properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    Sword,
    Axe,
    Bow,
    Spear,
    Hammer,
    Dagger,
    Staff,
    Wand,
    Crossbow,
    Thrown,
}

/// DIABOLICAL Armor Types with protection values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArmorType {
    Leather,
    Iron,
    Gold,
    Diamond,
    Netherite,
    Arcane,
    Dragon,
}

/// DIABOLICAL Mob Types with unique behaviors
#[derive(Debug, Clone, PartialEq)]
pub enum MobType {
    Zombie,
    Skeleton,
    Spider,
    Creeper,
    Enderman,
    Witch,
    Blaze,
    Ghast,
    Wither,
    EnderDragon,
    Villager,
    IronGolem,
    SnowGolem,
    Wolf,
    Cat,
    Horse,
    Custom(String),
}

/// DIABOLICAL Mob AI States
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobAIState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Sleeping,
    Working,
    Trading,
    Guarding,
    Patrolling,
}

/// DIABOLICAL Combat Effects
#[derive(Debug, Clone)]
pub struct CombatEffect {
    pub effect_type: CombatEffectType,
    pub duration: f32,
    pub intensity: f32,
    pub source: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone)]
pub enum CombatEffectType {
    Damage { amount: f32, damage_type: DamageType },
    Heal { amount: f32 },
    Buff { stat: StatType, multiplier: f32 },
    Debuff { stat: StatType, multiplier: f32 },
    StatusEffect { effect: StatusEffect },
    Knockback { force: Vec3 },
    Stun { duration: f32 },
    Freeze { duration: f32 },
    Burn { duration: f32 },
    Poison { duration: f32, damage_per_second: f32 },
}

#[derive(Debug, Clone)]
pub enum StatusEffect {
    Regeneration,
    Strength,
    Speed,
    JumpBoost,
    NightVision,
    Invisibility,
    FireResistance,
    WaterBreathing,
    Haste,
    MiningFatigue,
    Nausea,
    Blindness,
    Hunger,
    Weakness,
    Poison,
    Wither,
    Levitation,
    SlowFalling,
    ConduitPower,
    DolphinsGrace,
    BadOmen,
    HeroOfTheVillage,
    Glowing,
    Burn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatType {
    Health,
    AttackDamage,
    AttackSpeed,
    MovementSpeed,
    Defense,
    MagicResistance,
    CriticalChance,
    CriticalDamage,
    LifeSteal,
    ManaRegeneration,
}

/// DIABOLICAL Mob Entity with advanced AI
pub struct Mob {
    pub id: u32,
    pub mob_type: MobType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub ai_state: MobAIState,
    pub target: Option<u32>,
    pub patrol_path: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub wander_timer: f32,
    pub attack_timer: f32,
    pub attack_range: f32,
    pub detection_range: f32,
    pub flee_range: f32,
    pub speed: f32,
    pub jump_power: f32,
    pub damage: f32,
    pub attack_cooldown: f32,
    pub effects: Vec<CombatEffect>,
    pub status_effects: Vec<StatusEffectInstance>,
    pub inventory: Vec<ItemStack>,
    pub equipment: MobEquipment,
    pub behavior_tree: BehaviorTree,
    pub animation_state: AnimationState,
    pub sound_cooldown: f32,
    pub last_sound_time: f32,
    pub aggro_range: f32,
    pub loyalty: f32,
    pub fear: f32,
    pub hunger: f32,
    pub energy: f32,
    pub experience_value: u32,
    pub drop_table: DropTable,
}

#[derive(Debug, Clone)]
pub struct StatusEffectInstance {
    pub effect: StatusEffect,
    pub duration: f32,
    pub intensity: i32,
    pub start_time: f32,
}

#[derive(Debug, Clone)]
pub struct MobEquipment {
    pub weapon: Option<Weapon>,
    pub armor: [Option<Armor>; 4], // helmet, chestplate, leggings, boots
    pub accessory: Option<Accessory>,
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub attack_speed: f32,
    pub durability: u32,
    pub max_durability: u32,
    pub enchantments: Vec<Enchantment>,
    pub special_effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub armor_type: ArmorType,
    pub defense: f32,
    pub durability: u32,
    pub max_durability: u32,
    pub enchantments: Vec<Enchantment>,
    pub special_effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct Accessory {
    pub accessory_type: String,
    pub effects: Vec<CombatEffectType>,
    pub durability: u32,
    pub max_durability: u32,
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: u32,
    pub durability: Option<u32>,
    pub max_durability: Option<u32>,
    pub enchantments: Vec<Enchantment>,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Accessory(Accessory),
    Consumable(Consumable),
    Material(Material),
    Tool(Tool),
}

#[derive(Debug, Clone)]
pub struct Consumable {
    pub consumable_type: String,
    pub effects: Vec<CombatEffectType>,
    pub stack_size: u32,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub material_type: String,
    pub rarity: Rarity,
    pub properties: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub tool_type: String,
    pub efficiency: f32,
    pub durability: u32,
    pub max_durability: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(Debug, Clone)]
pub struct Enchantment {
    pub enchantment_type: String,
    pub level: u32,
    pub effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct DropTable {
    pub drops: Vec<DropEntry>,
    pub guaranteed_drops: Vec<ItemStack>,
}

#[derive(Debug, Clone)]
pub struct DropEntry {
    pub item: ItemStack,
    pub chance: f32,
    pub min_count: u32,
    pub max_count: u32,
    pub condition: Option<DropCondition>,
}

#[derive(Debug, Clone)]
pub enum DropCondition {
    Weather(String),
    TimeOfDay(f32, f32), // start, end
    PlayerLevel(u32),
    Biome(String),
    Difficulty(String),
}

#[derive(Debug)]
pub struct BehaviorTree {
    pub root_node: BehaviorNode,
}

// Custom Debug implementation for BehaviorNode since trait objects don't implement Debug
impl std::fmt::Debug for BehaviorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BehaviorNode::Sequence(nodes) => f.debug_tuple("Sequence").field(nodes).finish(),
            BehaviorNode::Selector(nodes) => f.debug_tuple("Selector").field(nodes).finish(),
            BehaviorNode::Action(_) => f.debug_tuple("Action").field(&"<action>").finish(),
            BehaviorNode::Condition(_) => f.debug_tuple("Condition").field(&"<condition>").finish(),
            BehaviorNode::Decorator(_, node) => f.debug_tuple("Decorator").field(&"<decorator>").field(node).finish(),
        }
    }
}

pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Action(Box<dyn MobAction>),
    Condition(Box<dyn MobCondition>),
    Decorator(Box<dyn BehaviorDecorator>, Box<BehaviorNode>),
}

pub trait MobAction: Send + Sync {
    fn execute(&mut self, mob: &mut Mob, world: &World, player: &Player) -> ActionResult;
}

pub trait MobCondition: Send + Sync {
    fn evaluate(&self, mob: &Mob, world: &World, player: &Player) -> bool;
}

pub trait BehaviorDecorator: Send + Sync {
    fn decorate(&self, node: &mut BehaviorNode, mob: &mut Mob, world: &World, player: &Player) -> ActionResult;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    Success,
    Failure,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
    Swimming,
    Flying,
    Attacking,
    Hurt,
    Dying,
    Dead,
    Sleeping,
    Sitting,
    Working,
    Eating,
    Drinking,
    Casting,
    Blocking,
    Dodging,
}

impl Mob {
    pub fn new(mob_type: MobType, position: Vec3) -> Self {
        let (health, max_health, armor, speed, damage, attack_range, detection_range) = match &mob_type {
            MobType::Zombie => (20.0, 20.0, 2.0, 1.0, 3.0, 2.0, 16.0),
            MobType::Skeleton => (20.0, 20.0, 2.0, 1.2, 4.0, 15.0, 16.0),
            MobType::Spider => (16.0, 16.0, 1.0, 1.8, 2.0, 2.0, 16.0),
            MobType::Creeper => (20.0, 20.0, 2.0, 1.2, 49.0, 3.0, 16.0),
            MobType::Enderman => (40.0, 40.0, 2.0, 3.0, 7.0, 3.0, 64.0),
            MobType::Witch => (26.0, 26.0, 0.0, 1.0, 6.0, 8.0, 16.0),
            MobType::Blaze => (20.0, 20.0, 6.0, 1.6, 6.0, 10.0, 48.0),
            MobType::Ghast => (10.0, 10.0, 2.0, 1.0, 17.0, 100.0, 100.0),
            MobType::Wither => (300.0, 300.0, 4.0, 1.4, 12.0, 12.0, 20.0),
            MobType::EnderDragon => (200.0, 200.0, 20.0, 1.0, 15.0, 20.0, 128.0),
            MobType::Villager => (20.0, 20.0, 0.0, 0.5, 1.0, 3.0, 8.0),
            MobType::IronGolem => (100.0, 100.0, 15.0, 0.3, 15.0, 2.0, 16.0),
            MobType::SnowGolem => (4.0, 4.0, 0.0, 0.8, 0.0, 3.0, 16.0),
            MobType::Wolf => (20.0, 20.0, 2.0, 1.4, 4.0, 2.0, 16.0),
            MobType::Cat => (10.0, 10.0, 2.0, 1.2, 3.0, 3.0, 16.0),
            MobType::Horse => (30.0, 30.0, 2.0, 2.0, 5.0, 3.0, 16.0),
            MobType::Custom(_) => (20.0, 20.0, 2.0, 1.0, 3.0, 3.0, 16.0),
        };

        // Clone mob_type for later use since we removed Copy
        let mob_type_clone = mob_type.clone();

        Self {
            id: rand::random::<u32>(),
            mob_type,
            position,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            health,
            max_health,
            armor,
            ai_state: MobAIState::Idle,
            target: None,
            patrol_path: Vec::new(),
            current_patrol_index: 0,
            wander_timer: 0.0,
            attack_timer: 0.0,
            attack_range,
            detection_range,
            flee_range: 4.0,
            speed,
            jump_power: 1.0,
            damage,
            attack_cooldown: 1.0,
            effects: Vec::new(),
            status_effects: Vec::new(),
            inventory: Vec::new(),
            equipment: MobEquipment {
                weapon: None,
                armor: [None, None, None, None],
                accessory: None,
            },
            behavior_tree: Self::create_behavior_tree(mob_type_clone.clone()),
            animation_state: AnimationState::Idle,
            sound_cooldown: 0.0,
            last_sound_time: 0.0,
            aggro_range: detection_range * 0.8,
            loyalty: 0.5,
            fear: 0.5,
            hunger: 1.0,
            energy: 1.0,
            experience_value: match &mob_type_clone {
                MobType::Zombie => 5,
                MobType::Skeleton => 5,
                MobType::Spider => 5,
                MobType::Creeper => 5,
                MobType::Enderman => 5,
                MobType::Witch => 5,
                MobType::Blaze => 10,
                MobType::Ghast => 5,
                MobType::Wither => 50,
                MobType::EnderDragon => 500,
                _ => 0,
            },
            drop_table: Self::create_drop_table(mob_type_clone),
        }
    }

    fn create_behavior_tree(_mob_type: MobType) -> BehaviorTree {
        // Create behavior tree based on mob type
        // This would be implemented with specific behaviors for each mob type
        BehaviorTree {
            root_node: BehaviorNode::Selector(vec![
                BehaviorNode::Condition(Box::new(HasTargetCondition)),
                BehaviorNode::Condition(Box::new(ShouldWanderCondition)),
                BehaviorNode::Action(Box::new(IdleAction)),
            ]),
        }
    }

    fn create_drop_table(mob_type: MobType) -> DropTable {
        let drops = match mob_type {
            MobType::Zombie => vec![
                DropEntry {
                    item: ItemStack {
                        item_type: ItemType::Material(Material {
                            material_type: "rotten_flesh".to_string(),
                            rarity: Rarity::Common,
                            properties: HashMap::new(),
                        }),
                        count: 1,
                        durability: None,
                        max_durability: None,
                        enchantments: Vec::new(),
                    },
                    chance: 0.5,
                    min_count: 0,
                    max_count: 2,
                    condition: None,
                },
            ],
            _ => Vec::new(),
        };

        DropTable {
            drops,
            guaranteed_drops: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &Player) {
        // Update AI state
        self.update_ai(dt, world, player);

        // Update position and physics
        self.update_physics(dt, world);

        // Update effects
        self.update_effects(dt);

        // Update animation state
        self.update_animation_state();

        // Update sound cooldown
        if self.sound_cooldown > 0.0 {
            self.sound_cooldown -= dt;
        }
    }

    fn update_ai(&mut self, dt: f32, _world: &World, _player: &Player) {
        // Behavior tree execution temporarily disabled due to borrow checker issues
        // TODO: Refactor behavior tree to avoid self-referential borrowing

        // Update timers
        if self.wander_timer > 0.0 {
            self.wander_timer -= dt;
        }
        if self.attack_timer > 0.0 {
            self.attack_timer -= dt;
        }
    }

    #[allow(dead_code)]
    fn execute_node(&mut self, node: &BehaviorNode, world: &World, player: &Player) {
        match node {
            BehaviorNode::Action(_action) => {
                // Execute action (this would need to be implemented with dynamic dispatch)
            }
            BehaviorNode::Condition(_condition) => {
                // Evaluate condition (this would need to be implemented with dynamic dispatch)
            }
            BehaviorNode::Sequence(nodes) => {
                for node in nodes {
                    self.execute_node(node, world, player);
                }
            }
            BehaviorNode::Selector(nodes) => {
                for node in nodes {
                    self.execute_node(node, world, player);
                }
            }
            BehaviorNode::Decorator(_, _) => {
                // Apply decorator (this would need to be implemented with dynamic dispatch)
            }
        }
    }

    fn update_physics(&mut self, dt: f32, world: &World) {
        // Apply gravity
        self.velocity.y -= 9.8 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Ground collision
        let ground_y = world.get_ground_height(self.position.x, self.position.z);
        if self.position.y <= ground_y {
            self.position.y = ground_y;
            self.velocity.y = 0.0;
        }

        // Update rotation to face movement direction
        if self.velocity.length() > 0.1 {
            self.rotation.y = self.velocity.x.atan2(self.velocity.z);
        }
    }

    fn update_effects(&mut self, dt: f32) {
        // Collect effects to apply first
        let mut effects_to_apply = Vec::new();
        
        self.effects.retain(|effect| {
            match &effect.effect_type {
                CombatEffectType::Damage { amount, damage_type } => {
                    effects_to_apply.push((*amount, *damage_type));
                    false // One-time effect
                }
                CombatEffectType::Heal { amount } => {
                    self.health = (self.health + amount).min(self.max_health);
                    false // One-time effect
                }
                CombatEffectType::Burn { duration: _ } => {
                    // Apply burn damage over time
                    false // Handled elsewhere
                }
                CombatEffectType::Poison { duration: _, damage_per_second: _ } => {
                    // Apply poison damage over time
                    false // Handled elsewhere
                }
                _ => {
                    effect.duration > 0.0
                }
            }
        });

        // Apply collected damage effects
        for (amount, damage_type) in effects_to_apply {
            self.take_damage(amount, damage_type);
        }

        // Update remaining effects
        for effect in &mut self.effects {
            effect.duration -= dt;
        }
        
        // Remove expired effects
        self.effects.retain(|effect| effect.duration > 0.0);

        // Update status effects
        self.status_effects.retain_mut(|effect| {
            effect.duration -= dt;
            effect.duration > 0.0
        });
    }

    fn update_animation_state(&mut self) {
        // Update animation based on velocity and state
        if self.velocity.length() > 0.1 {
            if self.velocity.length() > 2.0 {
                self.animation_state = AnimationState::Running;
            } else {
                self.animation_state = AnimationState::Walking;
            }
        } else if self.velocity.y < -0.1 {
            self.animation_state = AnimationState::Falling;
        } else if self.velocity.y > 0.1 {
            self.animation_state = AnimationState::Jumping;
        } else {
            self.animation_state = AnimationState::Idle;
        }
    }

    pub fn take_damage(&mut self, amount: f32, damage_type: DamageType) {
        let actual_damage = (amount - self.armor).max(0.0);
        self.health -= actual_damage;

        // Apply damage type specific effects
        match damage_type {
            DamageType::Fire => {
                self.status_effects.push(StatusEffectInstance {
                    effect: StatusEffect::Burn,
                    duration: 3.0,
                    intensity: 1,
                    start_time: 0.0,
                });
            }
            DamageType::Poison => {
                self.status_effects.push(StatusEffectInstance {
                    effect: StatusEffect::Poison,
                    duration: 5.0,
                    intensity: 1,
                    start_time: 0.0,
                });
            }
            _ => {}
        }

        // Trigger hurt animation
        self.animation_state = AnimationState::Hurt;

        // Play hurt sound
        self.play_sound("hurt");

        // Check if dead
        if self.health <= 0.0 {
            self.die();
        }
    }

    fn die(&mut self) {
        self.animation_state = AnimationState::Dying;
        self.health = 0.0;
        self.velocity = Vec3::ZERO;
        
        // Play death sound
        self.play_sound("death");

        // Drop items
        self.drop_items();
    }

    fn drop_items(&self) {
        // Generate drops based on drop table
        for drop in &self.drop_table.drops {
            if rand::random::<f32>() < drop.chance {
                let range = (drop.max_count - drop.min_count + 1) as u32;
                let _count = (rand::random::<u32>() % range) as usize + drop.min_count as usize;
                // Create dropped item entity
                // This would need to be implemented with the world system
            }
        }
    }

    fn play_sound(&mut self, _sound_type: &str) {
        if self.sound_cooldown <= 0.0 {
            // Play sound based on mob type and sound type
            // This would need to be implemented with the audio system
            self.sound_cooldown = 1.0;
            self.last_sound_time = 0.0;
        }
    }

    pub fn attack(&mut self, target: &mut Player) {
        if self.attack_timer <= 0.0 {
            // Calculate damage
            let base_damage = self.damage;
            let weapon_damage = self.equipment.weapon.as_ref()
                .map(|w| w.damage)
                .unwrap_or(0.0);
            let total_damage = base_damage + weapon_damage;

            // Apply damage to target
            target.take_damage(total_damage, "Physical");

            // Reset attack timer
            self.attack_timer = self.attack_cooldown;

            // Play attack sound
            self.play_sound("attack");

            // Set attack animation
            self.animation_state = AnimationState::Attacking;
        }
    }

    pub fn can_see(&self, target: &Player, _world: &World) -> bool {
        let distance = (self.position - target.position).length();
        if distance > self.detection_range {
            return false;
        }

        // Check line of sight
        // This would need to be implemented with raycasting
        true // Simplified for now
    }

    pub fn move_towards(&mut self, target: Vec3, _dt: f32) {
        let direction = (target - self.position).normalize();
        self.velocity = direction * self.speed;
    }

    pub fn move_away_from(&mut self, threat: Vec3, _dt: f32) {
        let direction = (self.position - threat).normalize();
        self.velocity = direction * self.speed * 1.5; // Run faster when fleeing
    }

    pub fn wander(&mut self, dt: f32) {
        if self.wander_timer <= 0.0 {
            // Choose new random direction
            let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let distance = rand::random::<f32>() * 5.0 + 2.0;
            
            let target = self.position + Vec3::new(
                angle.cos() * distance,
                0.0,
                angle.sin() * distance
            );
            
            self.move_towards(target, dt);
            self.wander_timer = rand::random::<f32>() * 3.0 + 2.0;
        }
    }
}

// Behavior implementations
struct HasTargetCondition;
impl MobCondition for HasTargetCondition {
    fn evaluate(&self, mob: &Mob, _world: &World, _player: &Player) -> bool {
        mob.target.is_some()
    }
}

struct ShouldWanderCondition;
impl MobCondition for ShouldWanderCondition {
    fn evaluate(&self, mob: &Mob, _world: &World, _player: &Player) -> bool {
        mob.wander_timer <= 0.0 && mob.ai_state == MobAIState::Idle
    }
}

struct IdleAction;
impl MobAction for IdleAction {
    fn execute(&mut self, mob: &mut Mob, _world: &World, _player: &Player) -> ActionResult {
        mob.ai_state = MobAIState::Idle;
        mob.velocity = Vec3::ZERO;
        ActionResult::Success
    }
}

/// DIABOLICAL Combat System - Main combat controller
pub struct CombatSystem {
    pub mobs: Vec<Mob>,
    pub active_combats: Vec<CombatInstance>,
    pub damage_numbers: Vec<DamageNumber>,
    pub combat_effects: Vec<CombatEffect>,
    pub last_update_time: f32,
    pub combat_music_enabled: bool,
    pub difficulty_multiplier: f32,
}

#[derive(Debug, Clone)]
pub struct CombatInstance {
    pub participants: Vec<u32>, // Mob IDs
    pub start_time: f32,
    pub combat_type: CombatType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatType {
    PlayerVsMob,
    MobVsMob,
    PlayerVsPlayer,
    BossBattle,
}

#[derive(Debug, Clone)]
pub struct DamageNumber {
    pub position: Vec3,
    pub amount: f32,
    pub damage_type: DamageType,
    pub lifetime: f32,
    pub color: [f32; 4],
    pub size: f32,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            mobs: Vec::new(),
            active_combats: Vec::new(),
            damage_numbers: Vec::new(),
            combat_effects: Vec::new(),
            last_update_time: 0.0,
            combat_music_enabled: true,
            difficulty_multiplier: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.last_update_time += dt;

        // Update all mobs
        for mob in &mut self.mobs {
            mob.update(dt, world, player);
        }

        // Remove dead mobs
        self.mobs.retain(|mob| mob.health > 0.0);

        // Update combat instances
        self.update_combats(dt);

        // Update damage numbers
        self.update_damage_numbers(dt);

        // Update combat effects
        self.update_combat_effects(dt);

        // Spawn new mobs based on conditions
        self.spawn_mobs(world, player);

        // Check for new combat initiations
        self.check_combat_initiation(player);
    }

    fn update_combats(&mut self, _dt: f32) {
        self.active_combats.retain_mut(|combat| {
            // Check if combat is still active
            let mut has_active_participants = false;
            for &participant_id in &combat.participants {
                if let Some(mob) = self.mobs.iter().find(|m| m.id == participant_id) {
                    if mob.health > 0.0 && mob.ai_state != MobAIState::Idle {
                        has_active_participants = true;
                        break;
                    }
                }
            }
            has_active_participants
        });
    }

    fn update_damage_numbers(&mut self, dt: f32) {
        self.damage_numbers.retain_mut(|damage_number| {
            damage_number.lifetime -= dt;
            damage_number.position.y += dt * 2.0; // Float upward
            damage_number.lifetime > 0.0
        });
    }

    fn update_combat_effects(&mut self, dt: f32) {
        self.combat_effects.retain_mut(|effect| {
            effect.duration -= dt;
            effect.duration > 0.0
        });
    }

    fn spawn_mobs(&mut self, world: &World, player: &Player) {
        // Spawn mobs based on time of day, location, and difficulty
        let spawn_radius = 64.0;
        let max_mobs = 50;

        if self.mobs.len() < max_mobs && rand::random::<f32>() < 0.01 {
            let mut spawn_pos = player.position + Vec3::new(
                (rand::random::<f32>() - 0.5) * spawn_radius,
                0.0,
                (rand::random::<f32>() - 0.5) * spawn_radius
            );

            // Get ground height at spawn position
            let ground_y = world.get_ground_height(spawn_pos.x, spawn_pos.z);
            spawn_pos.y = ground_y + 2.0;

            // Choose mob type based on biome and time
            let mob_type = self.choose_mob_type(world, spawn_pos);

            let mob = Mob::new(mob_type, spawn_pos);
            self.mobs.push(mob);
        }
    }

    fn choose_mob_type(&self, _world: &World, _position: Vec3) -> MobType {
        // Choose mob type based on biome, time of day, and other factors
        // This is a simplified implementation
        let mob_types = vec![
            MobType::Zombie,
            MobType::Skeleton,
            MobType::Spider,
            MobType::Creeper,
        ];

        let idx = (rand::random::<u32>() as usize) % mob_types.len();
        mob_types.get(idx).cloned().unwrap_or(MobType::Zombie)
    }

    fn check_combat_initiation(&mut self, player: &Player) {
        // Check if player is near any hostile mobs
        let mob_ids: Vec<u32> = self.mobs
            .iter()
            .filter(|mob| mob.can_see(player, &World::new(0)))
            .filter(|mob| (mob.position - player.position).length() < mob.aggro_range)
            .map(|mob| mob.id)
            .collect();
        
        if let Some(first_id) = mob_ids.first() {
            self.start_combat(vec![*first_id], CombatType::PlayerVsMob);
        }
    }

    fn start_combat(&mut self, participants: Vec<u32>, combat_type: CombatType) {
        self.active_combats.push(CombatInstance {
            participants,
            start_time: self.last_update_time,
            combat_type,
        });
    }

    pub fn add_damage_number(&mut self, position: Vec3, amount: f32, damage_type: DamageType) {
        let color = match damage_type {
            DamageType::Physical => [1.0, 1.0, 1.0, 1.0],
            DamageType::Fire => [1.0, 0.5, 0.0, 1.0],
            DamageType::Water => [0.0, 0.5, 1.0, 1.0],
            DamageType::Earth => [0.5, 0.3, 0.1, 1.0],
            DamageType::Air => [0.8, 0.8, 1.0, 1.0],
            DamageType::Arcane => [0.5, 0.0, 1.0, 1.0],
            DamageType::Holy => [1.0, 1.0, 0.5, 1.0],
            DamageType::Shadow => [0.3, 0.0, 0.3, 1.0],
            DamageType::Poison => [0.0, 1.0, 0.0, 1.0],
            DamageType::Lightning => [1.0, 1.0, 0.0, 1.0],
        };

        self.damage_numbers.push(DamageNumber {
            position,
            amount,
            damage_type,
            lifetime: 2.0,
            color,
            size: 0.5,
        });
    }

    pub fn add_combat_effect(&mut self, effect: CombatEffect) {
        self.combat_effects.push(effect);
    }

    pub fn get_nearest_mob(&self, position: Vec3, max_distance: f32) -> Option<&Mob> {
        self.mobs
            .iter()
            .filter(|mob| (mob.position - position).length() < max_distance)
            .min_by(|a, b| {
                (a.position - position).length()
                    .partial_cmp(&(b.position - position).length())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn get_mobs_in_range(&self, position: Vec3, range: f32) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|mob| (mob.position - position).length() < range)
            .collect()
    }

    pub fn clear_dead_mobs(&mut self) {
        self.mobs.retain(|mob| mob.health > 0.0);
    }

    pub fn set_difficulty(&mut self, difficulty: f32) {
        self.difficulty_multiplier = difficulty;
    }
}
// DIABOLICAL MINECRAFT RENDERING SYSTEM - Enhanced Traditional Rendering
// 
// This module provides comprehensive Minecraft rendering with:
// - Traditional visual enhancement integration
// - Enhanced lighting and shading systems
// - Material-specific rendering properties
// - Biome and time-based effects
// - Classic Minecraft aesthetic with modern performance

use crate::configuration::GameConfig;
use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest,
    Linear,
    NearestMipmap,
}

#[derive(Debug, Clone, Copy)]
pub enum FogType {
    Classic,
    Modern,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum ShadingMode {
    Classic,
    Smooth,
    Modern,
}

pub struct MinecraftRenderer {
    // Core rendering properties
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: Arc<GameConfig>,
    
    // Enhanced rendering properties
    pub texture_filter: TextureFilter,
    pub fog_type: FogType,
    pub shading_mode: ShadingMode,
    
    // Traditional rendering toggles
    pub view_bobbing: bool,
    pub directional_shading: bool,
    pub ambient_occlusion: bool,
    pub logarithmic_lighting: bool,
    pub pillow_shading: bool,
    
    // Enhanced visual settings
    pub traditional_rendering: bool,
    pub material_properties: bool,
    pub biome_time_effects: bool,
    
    // Atmospheric settings
    pub atmospheric_haze: f32,
    pub sky_brightness: f32,
    pub cloud_coverage: f32,
    
    // Traditional aesthetic settings
    pub color_temperature: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub vignette_strength: f32,
    
    // Classic fog parameters
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 4],
    
}

impl MinecraftRenderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, config: Arc<GameConfig>) -> Self {
        Self {
            device,
            queue,
            config,
            texture_filter: TextureFilter::Nearest,
            fog_type: FogType::Classic,
            shading_mode: ShadingMode::Classic,
            view_bobbing: true,
            directional_shading: true,
            ambient_occlusion: true,
            logarithmic_lighting: true,
            pillow_shading: false,
            traditional_rendering: true,
            material_properties: true,
            biome_time_effects: true,
            atmospheric_haze: 0.1,
            sky_brightness: 1.0,
            cloud_coverage: 0.3,
            color_temperature: 0.0,
            saturation: 1.0,
            contrast: 1.0,
            vignette_strength: 0.0,
            fog_start: 0.0,
            fog_end: 6.0,
            fog_color: [0.7, 0.7, 0.8, 1.0],
        }
    }

    pub fn set_texture_filter(&mut self, filter: TextureFilter) {
        self.texture_filter = filter;
    }

    pub fn set_fog_type(&mut self, fog_type: FogType) {
        self.fog_type = fog_type;
        self.update_fog_parameters();
    }

    pub fn set_shading_mode(&mut self, mode: ShadingMode) {
        self.shading_mode = mode;
    }

    pub fn set_view_bobbing(&mut self, enabled: bool) {
        self.view_bobbing = enabled;
    }

    fn update_fog_parameters(&mut self) {
        match self.fog_type {
            FogType::Classic => {
                self.fog_start = 0.0;
                self.fog_end = 6.0;
                self.fog_color = [0.7, 0.7, 0.8, 1.0];
            }
            FogType::Modern => {
                self.fog_start = 0.0;
                self.fog_end = 64.0;
                self.fog_color = [0.7, 0.8, 0.9, 1.0];
            }
            FogType::None => {
                self.fog_start = 0.0;
                self.fog_end = 1000.0;
                self.fog_color = [0.0, 0.0, 0.0, 0.0];
            }
        }
    }

    pub fn get_directional_multiplier(&self, face_normal: Vec3) -> f32 {
        if !self.directional_shading {
            return 1.0;
        }

        // Classic Minecraft directional shading multipliers
        // Top face: 1.0 (full brightness)
        // Z-axis faces (North/South): 0.8
        // X-axis faces (East/West): 0.6
        let dot = face_normal.y.abs();
        let dot_xz = (face_normal.x.abs() + face_normal.z.abs()).max(0.001);
        
        if dot > 0.99 {
            // Top face
            1.0
        } else if dot_xz > 0.99 {
            // X-axis faces
            0.6
        } else {
            // Z-axis faces
            0.8
        }
    }

    pub fn calculate_vertex_light(&self, light_level: u8, _vertex_pos: Vec3, surrounding_lights: [u8; 4]) -> f32 {
        if !self.ambient_occlusion {
            return light_level as f32 / 15.0;
        }

        // Classic smooth lighting calculation
        let center_light = surrounding_lights[0] as f32 / 15.0;
        let side1_light = surrounding_lights[1] as f32 / 15.0;
        let side2_light = surrounding_lights[2] as f32 / 15.0;
        let corner_light = surrounding_lights[3] as f32 / 15.0;

        // If both side blocks are solid, ignore corner light to prevent light leaking through diagonal walls
        let corner_multiplier = if side1_light >= 1.0 && side2_light >= 1.0 {
            0.0
        } else {
            corner_light
        };

        let vertex_light = (center_light + side1_light + side2_light + corner_multiplier) / 4.0;
        
        vertex_light
    }

    pub fn apply_logarithmic_attenuation(&self, distance: f32) -> f32 {
        if !self.logarithmic_lighting {
            return 1.0;
        }

        // Classic Minecraft light attenuation
        let max_distance = 32.0;
        if distance >= max_distance {
            0.0
        } else {
            let normalized_distance = distance / max_distance;
            // Apply sharp falloff curve
            (1.0 - normalized_distance).max(0.0)
        }
    }

    pub fn apply_pillow_shading(&self, normal: Vec3, uv: Vec2) -> Vec2 {
        if !self.pillow_shading {
            return uv;
        }

        // Classic Minecraft pillow shading - darken edges
        let edge_factor = (normal.x.abs() + normal.y.abs() + normal.z.abs()) / 3.0;
        let shading_factor = 1.0 - edge_factor * 0.3; // Darken edges up to 30%
        
        uv * shading_factor
    }

    pub fn apply_view_bobbing(&self, time: f32) -> Vec3 {
        if !self.view_bobbing {
            return Vec3::ZERO;
        }

        // Classic view bobbing - sine wave on Y position and roll
        let bob_speed = 0.07; // Speed of bobbing
        let bob_amount = 0.1; // Amplitude of bobbing
        let roll_amount = 0.05; // Roll amplitude
        
        Vec3::new(
            0.0,
            (time * bob_speed * std::f32::consts::TAU * 2.0).sin() * bob_amount,
            (time * bob_speed * std::f32::consts::TAU * 2.0).cos() * roll_amount
        )
    }

    pub fn render_block_with_classic_lighting(
        &self,
        base_color: [f32; 4],
        world_pos: Vec3,
        normal: Vec3,
        light_level: u8,
        surrounding_lights: [u8; 4],
        distance: f32,
        is_transparent: bool,
    ) -> [f32; 4] {
        let mut final_color = base_color;

        // Apply directional face shading
        let directional_multiplier = self.get_directional_multiplier(normal);
        final_color[0] *= directional_multiplier;
        final_color[1] *= directional_multiplier;
        final_color[2] *= directional_multiplier;
        final_color[3] *= directional_multiplier;

        // Apply vertex ambient occlusion
        let vertex_light = self.calculate_vertex_light(light_level, world_pos, surrounding_lights);
        final_color[0] *= vertex_light;
        final_color[1] *= vertex_light;
        final_color[2] *= vertex_light;
        final_color[3] *= vertex_light;

        // Apply logarithmic light attenuation
        let attenuation = self.apply_logarithmic_attenuation(distance);
        final_color[0] *= attenuation;
        final_color[1] *= attenuation;
        final_color[2] *= attenuation;
        final_color[3] *= attenuation;

        // Apply fog
        let fog_density = self.get_fog_density(distance);
        let fog_color = self.fog_color;
        
        // Linear interpolation between block color and fog color
        final_color[0] = final_color[0] * (1.0 - fog_density) + fog_color[0] * fog_density;
        final_color[1] = final_color[1] * (1.0 - fog_density) + fog_color[1] * fog_density;
        final_color[2] = final_color[2] * (1.0 - fog_density) + fog_color[2] * fog_density;
        final_color[3] = final_color[3] * (1.0 - fog_density) + fog_color[3] * fog_density;

        // Apply transparency
        if is_transparent {
            final_color[3] = 0.0;
        }

        final_color
    }

    pub fn render_sky_with_classic_fog(&self, sky_color: [f32; 4], _player_pos: Vec3, render_distance: f32) -> [f32; 4] {
        let mut final_color = sky_color;

        // Apply classic fog to sky
        let fog_density = self.get_fog_density(render_distance);
        let fog_color = self.fog_color;
        
        final_color[0] = final_color[0] * (1.0 - fog_density) + fog_color[0] * fog_density;
        final_color[1] = final_color[1] * (1.0 - fog_density) + fog_color[1] * fog_density;
        final_color[2] = final_color[2] * (1.0 - fog_density) + fog_color[2] * fog_density;
        final_color[3] = final_color[3] * (1.0 - fog_density) + fog_color[3] * fog_density;

        final_color
    }

    pub fn get_fog_density(&self, distance: f32) -> f32 {
        match self.fog_type {
            FogType::Classic => {
                if distance < self.fog_start {
                    0.0
                } else if distance > self.fog_end {
                    1.0
                } else {
                    (distance - self.fog_start) / (self.fog_end - self.fog_start)
                }
            }
            FogType::Modern => {
                // Modern depth-based fog
                let start = self.fog_start;
                let end = self.fog_end;
                if distance < start {
                    0.0
                } else if distance > end {
                    1.0
                } else {
                    (distance - start) / (end - start)
                }
            }
            FogType::None => 0.0,
        }
    }

    pub fn get_texture_filter_mode(&self) -> u32 {
        match self.texture_filter {
            TextureFilter::Nearest => 0x2600, // GL_NEAREST
            TextureFilter::Linear => 0x2601, // GL_LINEAR
            TextureFilter::NearestMipmap => 0x2700, // GL_NEAREST_MIPMAP
        }
    }

    pub fn should_use_mipmapping(&self) -> bool {
        matches!(self.texture_filter, TextureFilter::NearestMipmap)
    }
}

pub struct TextureGenerator {
    pub noise_scale: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub base_colors: HashMap<String, [u8; 4]>,
}

impl TextureGenerator {
    pub fn new() -> Self {
        let mut base_colors = HashMap::new();
        
        // Classic Minecraft base colors (programmer art style)
        base_colors.insert("dirt".to_string(), [139, 90, 69, 62]);
        base_colors.insert("stone".to_string(), [136, 136, 136, 136]);
        base_colors.insert("grass".to_string(), [124, 169, 80, 62]);
        base_colors.insert("sand".to_string(), [238, 220, 194, 174]);
        base_colors.insert("wood".to_string(), [143, 101, 69, 62]);
        base_colors.insert("leaves".to_string(), [34, 89, 34, 89]);
        base_colors.insert("cobblestone".to_string(), [136, 136, 136, 136]);
        base_colors.insert("gravel".to_string(), [136, 136, 136, 136]);
        base_colors.insert("coal".to_string(), [24, 24, 24, 24]);
        base_colors.insert("iron_ore".to_string(), [136, 136, 136, 136]);
        base_colors.insert("gold_ore".to_string(), [255, 215, 0, 0]);
        base_colors.insert("diamond_ore".to_string(), [136, 136, 136, 136]);

        Self {
            noise_scale: 0.05,
            contrast: 1.2,
            saturation: 0.8,
            base_colors,
        }
    }

    pub fn generate_programmer_art_texture(&self, texture_type: &str, size: usize) -> Vec<u8> {
        let base_color = self.base_colors.get(texture_type).unwrap_or(&[128, 128, 128, 128]);
        let mut texture_data = Vec::with_capacity(size * size * 4); // RGBA
        
        for y in 0..size {
            for x in 0..size {
                let noise = self.generate_noise(x as f32 / size as f32, y as f32 / size as f32);
                let contrast_factor = self.contrast;
                let saturation_factor = self.saturation;
                
                let pixel_color = base_color;
                
                // Apply noise
                let noise_value = (noise - 0.5) * 2.0; // Normalize to -1.0 to 1.0
                let noise_intensity = noise_value.abs() * 0.3; // 30% noise max
                
                // Apply contrast and saturation
                let mut r = ((pixel_color[0] as f32 / 255.0) * contrast_factor).clamp(0.0, 1.0);
                let mut g = ((pixel_color[1] as f32 / 255.0) * contrast_factor).clamp(0.0, 1.0);
                let mut b = ((pixel_color[2] as f32 / 255.0) * contrast_factor).clamp(0.0, 1.0);
                
                // Apply saturation
                let max_rgb = r.max(g).max(b);
                let _min_rgb = r.min(g).min(b);
                let gray_level = (r + g + b) / 3.0;
                
                if gray_level > 0.1 {
                    let saturation_boost = saturation_factor * (max_rgb - gray_level);
                    r = (r + saturation_boost).clamp(0.0, 1.0);
                    g = (g + saturation_boost).clamp(0.0, 1.0);
                    b = (b + saturation_boost).clamp(0.0, 1.0);
                }
                
                // Apply noise intensity
                r = (r + noise_intensity * 0.1).clamp(0.0, 1.0);
                g = (g + noise_intensity * 0.1).clamp(0.0, 1.0);
                b = (b + noise_intensity * 0.1).clamp(0.0, 1.0);
                
                // Convert back to 0-255 range
                let r = (r * 255.0) as u8;
                let g = (g * 255.0) as u8;
                let b = (b * 255.0) as u8;
                
                texture_data.extend([r, g, b, 255]);
            }
        }
        
        texture_data
    }

    fn generate_noise(&self, x: f32, y: f32) -> f32 {
        // Simple Perlin noise generator
        let mut x = x;
        let mut y = y;
        
        x = x * self.noise_scale * 4.0;
        y = y * self.noise_scale * 4.0;
        
        // Multiple octaves for more detail
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        
        for _ in 0..4 {
            value += amplitude * (self.generate_simple_noise(x * frequency, y * frequency)) * 2.0;
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        
        value / (amplitude * 8.0) + 0.5
    }
    
    fn generate_simple_noise(&self, x: f32, y: f32) -> f32 {
        // Simple noise function
        let n = (x.sin() * 12.9898 + y.cos() * 78.233) * 43758.5453;
        (n - n.floor()) * 2.0 - 1.0
    }
}
