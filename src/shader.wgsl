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
    @location(4) tex_index: u32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.ao = model.ao;
    out.depth = out.clip_position.w;
    
    let light_level = f32(model.light) / 15.0;
    out.light = max(0.1, light_level);
    out.tex_index = model.tex_index;
    out.tex_coords = model.tex_coords; // Pass raw world-scale coordinates for tiling
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // DIABOLICAL FRAGMENT TILING: Calculate atlas UVs here to support large greedy quads
    let atlas_size = 32.0; 
    let u_step = 1.0 / atlas_size;
    let v_step = 1.0 / atlas_size;
    let col = f32(in.tex_index % 32u);
    let row = f32(in.tex_index / 32u);

    // Apply fract() to tile within the 16x16 block texture
    let local_uv = fract(in.tex_coords);
    // Precision clamp to prevent atlas bleeding at edges (0.5 pixel margin)
    let margin = 0.5 / 512.0; 
    let u_clamped = clamp(local_uv.x, margin, 1.0 - margin);
    let v_clamped = clamp(local_uv.y, margin, 1.0 - margin);

    let atlas_uv = vec2<f32>(
        (col + u_clamped) * u_step,
        (row + v_clamped) * v_step
    );

    var base_color = textureSample(t_diffuse, s_diffuse, atlas_uv);
    if (base_color.a < 0.1) { discard; }
    
    // Proper Water Transparency
    if (in.tex_index == 9u) {
        base_color.a = 0.7;
    }
    
// --- VOXEL LIGHTING ---
    let brightness = in.light; 
    var lit_color = base_color.rgb * in.ao * brightness;
    
// DIABOLICAL CLOUD SHADOWS REMOVED: Caused flickering "dark circles" on blocks
    let _shadow_x = in.tex_coords.x * 100.0;

    // Simple Fog
    let fog_density = 0.015; 
    let fog_factor = 1.0 - exp(-in.depth * fog_density);
    lit_color = mix(lit_color, time_data.sky_color.rgb, clamp(fog_factor, 0.0, 1.0));

    // Underwater Tint
    if (time_data.underwater > 0.5) {
        lit_color = mix(lit_color, vec3<f32>(0.0, 0.2, 0.8), 0.4);
    }

    return vec4<f32>(lit_color, base_color.a);
}
// [ADD TO THE VERY END OF FILE]

@vertex
fn vs_ui(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // UI is 2D and uses raw NDC coordinates (-1.0 to 1.0), so we pass position directly
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.depth = 0.0;
    out.ao = 1.0;
    out.tex_index = model.tex_index;
    
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

@group(3) @binding(0) var<storage, read> chunks: array<ChunkCullData>;
@group(3) @binding(1) var<storage, read_write> draw_commands: array<DrawIndexedIndirect>;
@group(3) @binding(2) var<storage, read_write> draw_counter: atomic<u32>;

@compute @workgroup_size(64)
fn compute_cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&chunks)) { return; }

    let center = chunks[idx].pos_radius.xyz;
    let radius = chunks[idx].pos_radius.w;

// DIABOLICAL FRUSTUM EXTRACTION: Correctly pull planes from the View-Projection matrix
    let m = camera.view_proj;
    let p0 = vec4<f32>(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]);
    let p1 = vec4<f32>(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]);
    let p2 = vec4<f32>(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]);
    let p3 = vec4<f32>(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]);
    let p4 = vec4<f32>(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]);
    let p5 = vec4<f32>(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]);

    var visible = true;
    // Normalized Frustum Test
    if (dot(p0.xyz, center) + p0.w <= -radius) { visible = false; }
    if (dot(p1.xyz, center) + p1.w <= -radius) { visible = false; }
    if (dot(p2.xyz, center) + p2.w <= -radius) { visible = false; }
    if (dot(p3.xyz, center) + p3.w <= -radius) { visible = false; }
    if (dot(p4.xyz, center) + p4.w <= -radius) { visible = false; }
    if (dot(p5.xyz, center) + p5.w <= -radius) { visible = false; }

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