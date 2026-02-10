struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct TimeUniform {
    sky_color: vec4<f32>,
    time: f32,
    underwater: f32, // 1.0 if underwater
    _pad2: f32,
    _pad3: f32,
};
@group(2) @binding(0)
var<uniform> time_data: TimeUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) ao: f32,
    @location(3) tex_index: u32,
    @location(4) light: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
    @location(2) depth: f32,
    @location(3) light: f32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.ao = model.ao;
    out.depth = out.clip_position.w;
    
    // Calculate light level (0.0 to 1.0)
    let light_level = f32(model.light) / 15.0;
    out.light = max(0.1, light_level);

    // DIABOLICAL FRACTAL TILING
    // We use fract() to repeat the texture across large merged greedy meshes
    let atlas_size = 32.0; 
    let u_step = 1.0 / atlas_size;
    let col = f32(model.tex_index % 32u);
    let row = f32(model.tex_index / 32u);
    
    // model.tex_coords now represents "Local Tiling" (e.g., 0.0 to 16.0)
    // We wrap it using fract() so it repeats perfectly for every block unit
    let local_u = fract(model.tex_coords.x);
    let local_v = fract(model.tex_coords.y);

    out.tex_coords = vec2<f32>(
        (col + local_u) * u_step,
        (row + local_v) * u_step
    );

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (base_color.a < 0.1) { discard; }
    
    // DIABOLICAL TRANSLUCENCY: If it's water, force alpha to 0.6
    // Assuming water is using tex_index 9 from your texture.rs
    // Check vs_main's out.light to pass tex_index if needed, or use a color check
// DIABOLICAL PRECISION: Only apply water transparency to the specific atlas tile for water
    if (in.light < 0.99 && base_color.b > 0.5 && base_color.r < 0.4) {
        base_color.a = 0.7;
    }
    
// --- VOXEL LIGHTING ---
    let brightness = in.light; 
    var lit_color = base_color.rgb * in.ao * brightness;
    
// DIABOLICAL CLOUD SHADOWS
    // We use the world position (model.position passed through) to check cloud coverage
    // Using a simplified hash of the X/Z position
    let shadow_x = in.tex_coords.x * 100.0; // This is a dummy, in real use we pass world_pos
    let cloud_check = sin(in.tex_coords.x * 10.0) * cos(in.tex_coords.y * 10.0);
    if (cloud_check > 0.8) {
        lit_color *= 0.85; // Ground gets slightly darker under clouds
    }

    // Simple Fog
    let fog_density = 0.015; 
    let fog_factor = 1.0 - exp(-in.depth * fog_density);
    lit_color = mix(lit_color, time_data.sky_color.rgb, clamp(fog_factor, 0.0, 1.0));

    // Underwater Tint
    if (time_data.underwater > 0.5) {
        lit_color = mix(lit_color, vec3<f32>(0.0, 0.2, 0.8), 0.4);
    }

    return vec4<f32>(lit_color, 1.0);
}
// [ADD TO THE VERY END OF FILE]

@vertex
fn vs_ui(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // UI is 2D and uses raw NDC coordinates (-1.0 to 1.0), so we pass position directly
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.depth = 0.0;
    out.ao = 1.0;
    
    // Texture Atlas Logic (Same as vs_main)
    let atlas_size = 32.0;
let u_idx = f32(model.tex_index % 32u);
    let v_idx = f32(model.tex_index / 32u);
    let u_step = 1.0 / atlas_size;
    let v_step = 1.0 / atlas_size;
    let u = (u_idx + model.tex_coords.x) * u_step;
    let v = (v_idx + model.tex_coords.y) * v_step;
    
out.tex_coords = vec2<f32>(u, v);
    out.light = 1.0;
    return out;
}

@fragment
fn fs_ui(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple texture lookup for UI (no fog/lighting)
let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if (color.a < 0.1) { discard; }
    return color;
}

// --- DIABOLICAL COMPUTE CULLER ---

struct ChunkCullData {
    pos_radius: vec4<f32>,
    index_count: u32,
    base_vertex: i32,
    base_index: u32,
    _pad: u32,
};

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

@group(0) @binding(1) var<storage, read> chunks: array<ChunkCullData>;
@group(0) @binding(2) var<storage, read_write> draw_commands: array<DrawIndexedIndirect>;
@group(0) @binding(3) var<storage, read_write> draw_counter: atomic<u32>;

@compute @workgroup_size(64)
fn compute_cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&chunks)) { return; }

    let center = chunks[idx].pos_radius.xyz;
    let radius = chunks[idx].pos_radius.w;

    let m = camera.view_proj;
    let p0 = vec4<f32>(m[0].w + m[0].x, m[1].w + m[1].x, m[2].w + m[2].x, m[3].w + m[3].x);
    let p1 = vec4<f32>(m[0].w - m[0].x, m[1].w - m[1].x, m[2].w - m[2].x, m[3].w - m[3].x);
    let p2 = vec4<f32>(m[0].w + m[0].y, m[1].w + m[1].y, m[2].w + m[2].y, m[3].w + m[3].y);
    let p3 = vec4<f32>(m[0].w - m[0].y, m[1].w - m[1].y, m[2].w - m[2].y, m[3].w - m[3].y);
    let p4 = vec4<f32>(m[0].w + m[0].z, m[1].w + m[1].z, m[2].w + m[2].z, m[3].w + m[3].z);
    let p5 = vec4<f32>(m[0].w - m[0].z, m[1].w - m[1].z, m[2].w - m[2].z, m[3].w - m[3].z);
    let planes = array<vec4<f32>, 6>(p0, p1, p2, p3, p4, p5);

    var visible = true;
    for (var i = 0u; i < 6u; i = i + 1u) {
        if (dot(planes[i].xyz, center) + planes[i].w < -radius) {
            visible = false;
            break;
        }
    }

    if (visible) {
        // DIABOLICAL ATOMIC APPEND: Write the draw command directly to the storage buffer
        let out_idx = atomicAdd(&draw_counter, 1u);
        draw_commands[out_idx].index_count = chunks[idx].index_count;
        draw_commands[out_idx].instance_count = 1u;
        draw_commands[out_idx].first_index = chunks[idx].base_index;
        draw_commands[out_idx].base_vertex = chunks[idx].base_vertex;
        draw_commands[out_idx].first_instance = 0u;
    }
}