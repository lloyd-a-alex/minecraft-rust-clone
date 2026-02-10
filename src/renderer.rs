use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::world::{World, BlockPos, BlockType};
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

pub struct TextureRange {
    pub _tex_index: u32,
    pub index_start: u32,
    pub index_count: u32,
}

pub struct ChunkMesh { 
    pub vertex_buffer: Buffer, 
    pub index_buffer: Buffer, 
    pub ranges: Vec<TextureRange>,
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
    _ui_v_cache: Vec<Vertex>,
    _ui_i_cache: Vec<u32>,
    _ui_needs_update: bool,
    surface: Surface<'a>, device: Device, queue: Queue, pub config: SurfaceConfiguration,
    pipeline: RenderPipeline, ui_pipeline: RenderPipeline,
    depth_texture: TextureView, bind_group: BindGroup,
    camera_buffer: Buffer, camera_bind_group: BindGroup,
    time_buffer: Buffer, time_bind_group: BindGroup,
pub start_time: Instant, 
    chunk_meshes: HashMap<(i32, i32, i32), (ChunkMesh, u32)>, // (x, y, z) -> (Mesh, LOD_Level)
    entity_vertex_buffer: Buffer, entity_index_buffer: Buffer,
    pub break_progress: f32,
    
    // DIABOLICAL THREADING
    mesh_tx: Sender<(i32, i32, i32, u32, Arc<World>)>,
mesh_rx: Receiver<MeshTask>,
    pending_chunks: HashSet<(i32, i32, i32)>,

// DIABOLICAL GPU CULLING FIELDS
    compute_pipeline: ComputePipeline,
    chunk_data_buffer: Buffer,     
    indirect_draw_buffer: Buffer,   // Output: Draw Commands for the GPU
    indirect_count_buffer: Buffer,  // Total chunks to draw
    cull_bind_group: BindGroup,
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

        // RADICAL FIX: Single-threaded mesh generation to eliminate race conditions and coordinate chaos
        // The threaded worker had catastrophic bugs with dimensions and coordinate mapping
        let _worker_thread = {
            let t_rx = task_rx.clone();
            let r_tx = result_tx.clone();
            std::thread::spawn(move || {
                while let Ok((cx, cy, cz, lod, world)) = t_rx.recv() {
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
                        let step = 1 << lod; // LOD 0 = 1, LOD 1 = 2, LOD 2 = 4
                        
                        // RADICAL FIX: Consistent 16x16x16 chunk dimensions for ALL axes
                        for axis in 0..3 {
                            let (dims_u, dims_v) = (16usize, 16usize);
                            let dims_main = 16usize;
                            
                            for d in (0..dims_main).step_by(step as usize) {
                                for dir in 0..2 {
                                    let face_id = match axis { 
                                        0 => if dir==0 {0} else {1}, // Y: Top/Bottom
                                        1 => if dir==0 {2} else {3}, // X: Right/Left  
                                        _ => if dir==0 {4} else {5}  // Z: Front/Back
                                    };
                                    
                                    let mut mask = vec![BlockType::Air; dims_u * dims_v];
                                    
                                    // Build mask for this slice
                                    for u in (0..dims_u).step_by(step as usize) {
                                        for v in (0..dims_v).step_by(step as usize) {
                                            // RADICAL FIX: Consistent coordinate mapping
                                            let (lx, ly, lz) = match axis {
                                                0 => (u, d, v),      // Y-axis slice: x=u, y=d, z=v
                                                1 => (d, u, v),      // X-axis slice: x=d, y=u, z=v
                                                _ => (u, v, d)       // Z-axis slice: x=u, y=v, z=d
                                            };
                                            
                                            if lx >= 16 || ly >= 16 || lz >= 16 {
                                                continue; // Safety bounds check
                                            }
                                            
                                            let blk = chunk.get_block(lx, ly, lz);
                                            if !blk.is_solid() {
                                                continue;
                                            }
                                            
                                            // Check neighbor visibility
                                            let (nx, ny, nz) = match face_id { 
                                                0 => (lx as i32, ly as i32 + 1, lz as i32), // Top
                                                1 => (lx as i32, ly as i32 - 1, lz as i32), // Bottom
                                                2 => (lx as i32 + 1, ly as i32, lz as i32), // Right
                                                3 => (lx as i32 - 1, ly as i32, lz as i32), // Left
                                                4 => (lx as i32, ly as i32, lz as i32 + 1), // Front
                                                5 => (lx as i32, ly as i32, lz as i32 - 1), // Back
                                                _ => (0, 0, 0)
                                            };
                                            
                                            // RADICAL FIX: Check neighbor in current chunk or adjacent chunks
                                            let neighbor = if nx >= 0 && nx < 16 && ny >= 0 && ny < 16 && nz >= 0 && nz < 16 {
                                                chunk.get_block(nx as usize, ny as usize, nz as usize)
                                            } else {
                                                // Check world neighbor - simplified, treat as air for visibility
                                                BlockType::Air
                                            };
                                            
                                            let visible = !neighbor.is_solid() || (neighbor.is_transparent() && neighbor != blk);
                                            if visible { 
                                                mask[(v / step as usize) * (dims_u / step as usize) + (u / step as usize)] = blk; 
                                            }
                                        }
                                    }
                                    
                                    // Greedy meshing on the mask
                                    let mask_w = dims_u / step as usize;
                                    let mask_h = dims_v / step as usize;
                                    let mut n = 0;
                                    
                                    while n < mask_w * mask_h {
                                        let blk = mask[n];
                                        if blk != BlockType::Air {
                                            // Find width
                                            let mut w = 1;
                                            while (n % mask_w + w) < mask_w && mask[n + w] == blk {
                                                w += 1;
                                            }
                                            
                                            // Find height
                                            let mut h = 1;
                                            'h_loop: while (n / mask_w + h) < mask_h {
                                                for k in 0..w {
                                                    if mask[n + k + h * mask_w] != blk { 
                                                        break 'h_loop; 
                                                    }
                                                }
                                                h += 1;
                                            }
                                            
                                            // Calculate world position
                                            let mask_u = n % mask_w;
                                            let mask_v = n / mask_w;
                                            let world_u = (mask_u * step as usize) as f32;
                                            let world_v = (mask_v * step as usize) as f32;
                                            let world_d = d as f32;
                                            let world_w = (w * step as usize) as f32;
                                            let world_h = (h * step as usize) as f32;
                                            
                                            // RADICAL FIX: Consistent world coordinate generation
                                            let (wx, wy, wz) = match axis {
                                                0 => (bx + world_u, by + world_d + 1.0, bz + world_v), // Y-face at top
                                                1 => (bx + world_d + 1.0, by + world_u, bz + world_v), // X-face at right
                                                _ => (bx + world_u, by + world_v, bz + world_d + 1.0)  // Z-face at front
                                            };
                                            
                                            // Generate face with correct orientation
                                            let tex_index = match face_id { 
                                                0 => blk.get_texture_top(), 
                                                1 => blk.get_texture_bottom(), 
                                                _ => blk.get_texture_side() 
                                            };
                                            
                                            // RADICAL FIX: Correct face vertex generation with proper winding
                                            let positions = match face_id {
                                                0 => [ // Top face (Y+), CCW when looking down
                                                    [wx, wy, wz + world_h],
                                                    [wx + world_w, wy, wz + world_h],
                                                    [wx + world_w, wy, wz],
                                                    [wx, wy, wz],
                                                ],
                                                1 => [ // Bottom face (Y-), CCW when looking up
                                                    [wx, wy - 1.0, wz],
                                                    [wx + world_w, wy - 1.0, wz],
                                                    [wx + world_w, wy - 1.0, wz + world_h],
                                                    [wx, wy - 1.0, wz + world_h],
                                                ],
                                                2 => [ // Right face (X+), CCW when looking left
                                                    [wx, wy - world_w, wz + world_h],
                                                    [wx, wy, wz + world_h],
                                                    [wx, wy, wz],
                                                    [wx, wy - world_w, wz],
                                                ],
                                                3 => [ // Left face (X-), CCW when looking right
                                                    [wx - 1.0, wy, wz],
                                                    [wx - 1.0, wy, wz + world_h],
                                                    [wx - 1.0, wy - world_w, wz + world_h],
                                                    [wx - 1.0, wy - world_w, wz],
                                                ],
                                                4 => [ // Front face (Z+), CCW when looking back
                                                    [wx + world_w, wy, wz],
                                                    [wx, wy, wz],
                                                    [wx, wy + world_h, wz],
                                                    [wx + world_w, wy + world_h, wz],
                                                ],
                                                5 => [ // Back face (Z-), CCW when looking forward
                                                    [wx, wy, wz - 1.0],
                                                    [wx + world_w, wy, wz - 1.0],
                                                    [wx + world_w, wy + world_h, wz - 1.0],
                                                    [wx, wy + world_h, wz - 1.0],
                                                ],
                                                _ => [[0.0; 3]; 4],
                                            };
                                            
                                            let base_i = i_cnt;
                                            vertices.extend_from_slice(&[
                                                Vertex { position: positions[0], tex_coords: [0.0, world_h], ao: 1.0, tex_index, light: 15.0 },
                                                Vertex { position: positions[1], tex_coords: [world_w, world_h], ao: 1.0, tex_index, light: 15.0 },
                                                Vertex { position: positions[2], tex_coords: [world_w, 0.0], ao: 1.0, tex_index, light: 15.0 },
                                                Vertex { position: positions[3], tex_coords: [0.0, 0.0], ao: 1.0, tex_index, light: 15.0 },
                                            ]);
                                            
                                            // RADICAL FIX: Correct triangle winding order
                                            indices.extend_from_slice(&[
                                                base_i, base_i + 1, base_i + 2,
                                                base_i, base_i + 2, base_i + 3
                                            ]);
                                            
                                            i_cnt += 4;
                                            
                                            // Clear mask
                                            for l in 0..h { 
                                                for k in 0..w { 
                                                    mask[n + k + l * mask_w] = BlockType::Air; 
                                                } 
                                            }
                                        }
                                        n += 1;
                                    }
                                }
                            }
                        }
                    }


                    // RADICAL FIX: Proper range batching
                    let mut final_ranges = Vec::new();
                    if !indices.is_empty() {
                        final_ranges.push(TextureRange { 
                            _tex_index: 0, 
                            index_start: 0, 
                            index_count: indices.len() as u32 
                        });
                    }

                    let _ = r_tx.send(MeshTask { 
                        cx, cy, cz, lod, 
                        vertices, 
                        indices, 
                        ranges: final_ranges 
                    });
                }
            });
        }


        Self { 
            particles: Vec::new(), _ui_v_cache: Vec::new(), _ui_i_cache: Vec::new(), _ui_needs_update: true, surface, device, queue, config, pipeline, ui_pipeline, depth_texture, bind_group, camera_bind_group, camera_buffer, time_bind_group, time_buffer, start_time: Instant::now(), 
            chunk_meshes: HashMap::new(),
            entity_vertex_buffer, entity_index_buffer, 
            break_progress: 0.0,
mesh_tx: task_tx,
            mesh_rx: result_rx,
pending_chunks: HashSet::new(),
            compute_pipeline,
            chunk_data_buffer,
            indirect_draw_buffer,
            indirect_count_buffer,
            cull_bind_group,
        }
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
        for (key, _) in &world.chunks { self.update_chunk(key.0, key.1, key.2, world); }
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
            self.chunk_meshes.insert((999, 999, 999), (ChunkMesh { vertex_buffer: vb, index_buffer: ib, ranges: vec![TextureRange { _tex_index: 228, index_start: 0, index_count: indices.len() as u32 }], total_indices: indices.len() as u32 }, 0));
        }
    }

    // RADICAL FIX: Completely rewritten update_chunk with proper coordinate handling
    pub fn update_chunk(&mut self, cx: i32, cy: i32, cz: i32, world: &World) {
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
                                
                                // RADICAL FIX: Proper neighbor checking with world lookup
                                let neighbor = if nx >= 0 && nx < 16 && ny >= 0 && ny < 16 && nz >= 0 && nz < 16 {
                                    chunk.get_block(nx as usize, ny as usize, nz as usize)
                                } else {
                                    // Look up in world for proper boundary culling
                                    let wx = cx * 16 + nx;
                                    let wy = cy * 16 + ny;
                                    let wz = cz * 16 + nz;
                                    world.get_block(BlockPos { x: wx, y: wy, z: wz })
                                };

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

                                self.add_face_greedy_fixed(&mut chunk_v, &mut chunk_i, &mut i_cnt, wx, wy, wz, world_w, world_h, face_id, blk);

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
                    ranges: Vec::new(), 
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
// DIABOLICAL GREEDY MESHER HELPER: Restored absolute mathematical alignment
    fn add_face_greedy(&self, v: &mut Vec<Vertex>, i: &mut Vec<u32>, i_count: &mut u32, x: f32, y: f32, z: f32, w: f32, h: f32, face: usize, block: BlockType) {
        let tex_index = match face {
            0 => block.get_texture_top(),
            1 => block.get_texture_bottom(),
            _ => block.get_texture_side()
        };
        
        // w and h are the greedy dimensions in the plane of the face
        let (u0, v0, u1, v1) = (0.0, 0.0, w, h);

        let positions = match face {
            0 => [[x, y + 1.0, z], [x, y + 1.0, z + h], [x + w, y + 1.0, z + h], [x + w, y + 1.0, z]], // Top (XZ plane)
            1 => [[x, y, z + h], [x, y, z], [x + w, y, z], [x + w, y, z + h]], // Bottom (XZ plane)
            2 => [[x + 1.0, y, z], [x + 1.0, y + w, z], [x + 1.0, y + w, z + h], [x + 1.0, y, z + h]], // Right (YZ plane)
            3 => [[x, y, z + h], [x, y + w, z + h], [x, y + w, z], [x, y, z]], // Left (YZ plane)
            4 => [[x, y, z + 1.0], [x + w, y, z + 1.0], [x + w, y + h, z + 1.0], [x, y + h, z + 1.0]], // Front (XY plane)
            5 => [[x + w, y, z], [x, y, z], [x, y + h, z], [x + w, y + h, z]], // Back (XY plane)
            _ => [[0.0; 3]; 4],
        };

        let base_i = *i_count;
        v.extend_from_slice(&[
            Vertex { position: positions[0], tex_coords: [u0, v1], ao: 1.0, tex_index, light: 15.0 },
            Vertex { position: positions[1], tex_coords: [u0, v0], ao: 1.0, tex_index, light: 15.0 },
            Vertex { position: positions[2], tex_coords: [u1, v0], ao: 1.0, tex_index, light: 15.0 },
            Vertex { position: positions[3], tex_coords: [u1, v1], ao: 1.0, tex_index, light: 15.0 },
        ]);
        i.extend_from_slice(&[base_i, base_i + 1, base_i + 2, base_i + 2, base_i + 3, base_i]);
        *i_count += 4;
    }

fn add_ui_quad(&self, uv: &mut Vec<Vertex>, ui: &mut Vec<u32>, uoff: &mut u32, x: f32, y: f32, w: f32, h: f32, tex_index: u32) {
        uv.push(Vertex{position:[x,y+h,0.0], tex_coords:[0.0,0.0], ao:1.0, tex_index, light: 1.0}); uv.push(Vertex{position:[x+w,y+h,0.0], tex_coords:[1.0,0.0], ao:1.0, tex_index, light: 1.0});
        uv.push(Vertex{position:[x+w,y,0.0], tex_coords:[1.0,1.0], ao:1.0, tex_index, light: 1.0}); uv.push(Vertex{position:[x,y,0.0], tex_coords:[0.0,1.0], ao:1.0, tex_index, light: 1.0});
        ui.push(*uoff); ui.push(*uoff+1); ui.push(*uoff+2); ui.push(*uoff); ui.push(*uoff+2); ui.push(*uoff+3); *uoff += 4;
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
            let tex_id = 2u32; // CLASSIC DIRT BACKGROUND
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
        rpass.set_bind_group(0, &self.bind_group, &[]);
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
        // DIABOLICAL MESH SYNC & LOD MANAGEMENT
        while let Ok(task) = self.mesh_rx.try_recv() {
            self.pending_chunks.remove(&(task.cx, task.cy, task.cz));
if task.vertices.is_empty() {
                self.chunk_meshes.remove(&(task.cx, task.cy, task.cz));
            } else {
                let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk VB"), contents: bytemuck::cast_slice(&task.vertices), usage: BufferUsages::VERTEX });
                let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk IB"), contents: bytemuck::cast_slice(&task.indices), usage: BufferUsages::INDEX });
                self.chunk_meshes.insert((task.cx, task.cy, task.cz), (ChunkMesh { 
                    vertex_buffer: vb, 
                    index_buffer: ib, 
                    ranges: task.ranges, 
                    total_indices: task.indices.len() as u32 
                }, task.lod));
            }
        }

let p_cx = (player.position.x / 16.0).floor() as i32;
        let _p_cy = (player.position.y / 16.0).floor() as i32;
        let p_cz = (player.position.z / 16.0).floor() as i32;
        let world_arc = Arc::new(world.clone());

        // MASSIVE RENDER DISTANCE: 32 Chunks (512 blocks) with LOD
        let render_dist = 32;
        for dx in -render_dist..=render_dist {
            for dz in -render_dist..=render_dist {
                for dy in 0..8 { // 8 vertical chunks (128 height)
                    let target = (p_cx + dx, dy, p_cz + dz);
                    let dist_sq = (dx*dx + dz*dz) as f32;
                    
                    let target_lod = if dist_sq > 256.0 { 2 } else if dist_sq > 64.0 { 1 } else { 0 };

                    if !self.pending_chunks.contains(&target) {
                        let current_lod = self.chunk_meshes.get(&target).map(|m| m.1);
                        let needs_update = world.chunks.get(&target).map(|c| c.mesh_dirty).unwrap_or(false);
                        
                        if needs_update || current_lod != Some(target_lod) {
                            if world.chunks.contains_key(&target) {
                                self.pending_chunks.insert(target);
                                let _ = self.mesh_tx.send((target.0, target.1, target.2, target_lod, world_arc.clone()));
                            }
                        }
                    }
                }
            }
        }

// DIABOLICAL MESH SYNC
        while let Ok(task) = self.mesh_rx.try_recv() {
            self.pending_chunks.remove(&(task.cx, task.cy, task.cz));
            if task.vertices.is_empty() {
                self.chunk_meshes.remove(&(task.cx, task.cy, task.cz));
            } else {
                let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk VB"), contents: bytemuck::cast_slice(&task.vertices), usage: BufferUsages::VERTEX });
                let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Chunk IB"), contents: bytemuck::cast_slice(&task.indices), usage: BufferUsages::INDEX });
                self.chunk_meshes.insert((task.cx, task.cy, task.cz), (ChunkMesh { vertex_buffer: vb, index_buffer: ib, ranges: task.ranges, total_indices: task.indices.len() as u32 }, task.lod));
            }
        }

        let world_arc = Arc::new(world.clone());
        let p_cx = (player.position.x / 16.0).floor() as i32;
        let p_cy = (player.position.y / 16.0).floor() as i32;
        let p_cz = (player.position.z / 16.0).floor() as i32;
        
        unsafe {
            let world_mut = (world as *const World as *mut World).as_mut().unwrap();
            world_mut.update_occlusion(p_cx, p_cy, p_cz);
        }

        for dx in -8..=8 {
            for dz in -8..=8 {
                for dy in 0..8 {
                    let target = (p_cx + dx, dy, p_cz + dz);
                    if !self.pending_chunks.contains(&target) {
                        let needs_update = world.chunks.get(&target).map(|c| c.mesh_dirty).unwrap_or(false);
                        if needs_update {
                            self.pending_chunks.insert(target);
                            let _ = self.mesh_tx.send((target.0, target.1, target.2, 0, world_arc.clone()));
                        }
                    }
                }
            }
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let view_proj = player.build_view_projection_matrix(self.config.width as f32 / self.config.height as f32);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
let time = self.start_time.elapsed().as_secs_f32();
        let eye_bp = BlockPos { x: player.position.x.floor() as i32, y: (player.position.y + player.height * 0.4).floor() as i32, z: player.position.z.floor() as i32 };
let is_underwater = if world.get_block(eye_bp).is_water() { 1.0f32 } else { 0.0f32 };
        
// BIOME FOG LOGIC
        let noise_gen = crate::noise_gen::NoiseGenerator::new(world.seed); 
        let (cont, eros, _weird, temp) = noise_gen.get_height_params(eye_bp.x, eye_bp.z);
        let humid = noise_gen.get_noise_octaves(eye_bp.x as f64 * 0.01, 44.0, eye_bp.z as f64 * 0.01, 3) as f32;
        let biome = noise_gen.get_biome(cont, eros, temp, humid, eye_bp.y);
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
pass.set_pipeline(&self.pipeline); 
            pass.set_bind_group(0, &self.bind_group, &[]); 
            pass.set_bind_group(1, &self.camera_bind_group, &[]); 
            pass.set_bind_group(2, &self.time_bind_group, &[]);

// --- DIABOLICAL GPU COMPUTE CULLING & INDIRECT PASS ---
            // Note: For true Indirect Drawing, we must consolidate all chunk geometry into one MEGA-BUFFER.
            // Since we use separate buffers, we will use multi-draw indirect for each range.
            let mut cull_data = Vec::with_capacity(self.chunk_meshes.len());
            for (&(cx, cy, cz), (mesh, _)) in &self.chunk_meshes {
                if cx == 999 { continue; }
                if world.occluded_chunks.contains(&(cx, cy, cz)) { continue; }
                
                for range in &mesh.ranges {
                    cull_data.push(ChunkCullData { 
                        pos: [(cx * 16 + 8) as f32, (cy * 16 + 8) as f32, (cz * 16 + 8) as f32, 14.0],
                        index_count: range.index_count,
                        base_vertex: 0,
                        base_index: range.index_start,
                        _pad: 0,
                    });
                }
            }

            if !cull_data.is_empty() {
                // 1. Update Input Data
                self.queue.write_buffer(&self.chunk_data_buffer, 0, bytemuck::cast_slice(&cull_data));
                self.queue.write_buffer(&self.indirect_count_buffer, 0, bytemuck::cast_slice(&[0u32; 1])); // Reset atomic counter

// 2. Compute Pass (GPU decides what to draw)
                let mut c_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Cull Encoder") });
                {
                    let mut cpass = c_encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("Cull Pass"), timestamp_writes: None });
                    cpass.set_pipeline(&self.compute_pipeline);
                    // DIABOLICAL BIND GROUP SYNC: All indices must be bound to satisfy the Pipeline Layout
                    cpass.set_bind_group(0, &self.bind_group, &[]); 
                    cpass.set_bind_group(1, &self.camera_bind_group, &[]);
                    cpass.set_bind_group(2, &self.time_bind_group, &[]);
                    cpass.set_bind_group(3, &self.cull_bind_group, &[]);
                    cpass.dispatch_workgroups((cull_data.len() as u32 + 63) / 64, 1, 1);
                }
                self.queue.submit(std::iter::once(c_encoder.finish()));

                // 3. Render Pass (GPU draws exactly what it calculated)
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
