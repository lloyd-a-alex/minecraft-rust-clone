// Classic Minecraft Vertex Shader
// Implements authentic Java Edition rendering with nearest-neighbor filtering
// and precise directional face shading

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) light_level: u32,
    @location(5) surrounding_lights: vec4<u32>,
    @location(6) world_pos: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_color: vec4<f32>,
    @location(1) frag_uv: vec2<f32>,
    @location(2) frag_normal: vec3<f32>,
    @location(3) frag_light_level: u32,
    @location(4) frag_surrounding_lights: vec4<u32>,
    @location(5) frag_world_pos: vec3<f32>,
    @location(6) frag_distance: f32,
};

struct Uniforms {
    @group(0) @binding(0) model_matrix: mat4x4<f32>,
    @group(0) @binding(1) view_matrix: mat4x4<f32>,
    @group(0) @binding(2) projection_matrix: mat4x4<f32>,
    @group(0) @binding(3) camera_pos: vec3<f32>,
    @group(0) @binding(4) time: f32,
    @group(0) @binding(5) view_bobbing: f32,
    @group(0) @binding(6) fog_start: f32,
    @group(0) @binding(7) fog_end: f32,
    @group(0) @binding(8) fog_color: vec4<f32>,
    @group(0) @binding(9) directional_shading: f32,
    @group(0) @binding(10) ambient_occlusion: f32,
    @group(0) @binding(11) logarithmic_lighting: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Apply view bobbing to camera position
    var camera_pos = uniforms.camera_pos;
    if uniforms.view_bobbing > 0.5 {
        let bob_speed = 0.07;
        let bob_amount = 0.1;
        let roll_amount = 0.05;
        
        camera_pos.y += sin(uniforms.time * bob_speed * 6.28318530718 * 2.0) * bob_amount;
        camera_pos.z += cos(uniforms.time * bob_speed * 6.28318530718 * 2.0) * roll_amount;
    }
    
    // Transform position
    let world_position = input.world_pos;
    let view_position = world_position - camera_pos;
    output.frag_world_pos = world_position;
    output.frag_distance = length(view_position);
    
    // Calculate clip position
    let model_pos = uniforms.model_matrix * vec4<f32>(input.position, 1.0);
    let view_pos = uniforms.view_matrix * model_pos;
    output.clip_position = uniforms.projection_matrix * view_pos;
    
    // Pass through other data
    output.frag_color = input.color;
    output.frag_uv = input.uv;
    output.frag_normal = input.normal;
    output.frag_light_level = input.light_level;
    output.frag_surrounding_lights = input.surrounding_lights;
    
    return output;
}

// Classic Minecraft Fragment Shader
// Implements authentic Java Edition lighting with directional face shading,
// vertex ambient occlusion, and logarithmic light attenuation

@group(1) @binding(0)
var texture_sampler: sampler;

@group(1) @binding(1)
var texture_atlas: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture with nearest-neighbor filtering
    let tex_color = textureSample(texture_atlas, texture_sampler, input.frag_uv);
    
    // Apply directional face shading
    var final_color = tex_color;
    if uniforms.directional_shading > 0.5 {
        let directional_multiplier = get_directional_multiplier(input.frag_normal);
        final_color = final_color * directional_multiplier;
    }
    
    // Apply vertex ambient occlusion
    if uniforms.ambient_occlusion > 0.5 {
        let vertex_light = calculate_vertex_light(
            input.frag_light_level,
            input.frag_world_pos,
            input.frag_surrounding_lights
        );
        final_color = final_color * vertex_light;
    }
    
    // Apply logarithmic light attenuation
    if uniforms.logarithmic_lighting > 0.5 {
        let attenuation = apply_logarithmic_attenuation(input.frag_distance);
        final_color = final_color * attenuation;
    }
    
    // Apply classic fog
    let fog_density = get_fog_density(input.frag_distance);
    let fog_color = uniforms.fog_color;
    
    // Linear interpolation between block color and fog color
    final_color = mix(final_color, fog_color, fog_density);
    
    return final_color;
}

// Helper functions for classic Minecraft rendering

fn get_directional_multiplier(normal: vec3<f32>) -> f32 {
    // Classic Minecraft directional shading multipliers
    // Top face: 1.0 (full brightness)
    // Z-axis faces (North/South): 0.8
    // X-axis faces (East/West): 0.6
    let dot = abs(normal.y);
    let dot_xz = max(abs(normal.x) + abs(normal.z), 0.001);
    
    if dot > 0.99 {
        // Top face
        return 1.0;
    } else if dot_xz > 0.99 {
        // X-axis faces
        return 0.6;
    } else {
        // Z-axis faces
        return 0.8;
    }
}

fn calculate_vertex_light(light_level: u32, vertex_pos: vec3<f32>, surrounding_lights: vec4<u32>) -> f32 {
    // Classic smooth lighting calculation
    let center_light = f32(surrounding_lights[0]) / 15.0;
    let side1_light = f32(surrounding_lights[1]) / 15.0;
    let side2_light = f32(surrounding_lights[2]) / 15.0;
    let corner_light = f32(surrounding_lights[3]) / 15.0;
    
    // If both side blocks are solid, ignore corner light to prevent light leaking through diagonal walls
    let corner_multiplier = if side1_light >= 1.0 && side2_light >= 1.0 {
        0.0
    } else {
        corner_light
    };
    
    let vertex_light = (center_light + side1_light + side2_light + corner_multiplier) / 4.0;
    return vertex_light;
}

fn apply_logarithmic_attenuation(distance: f32) -> f32 {
    // Classic Minecraft light attenuation
    let max_distance = 32.0;
    if distance >= max_distance {
        return 0.0;
    } else {
        let normalized_distance = distance / max_distance;
        // Apply sharp falloff curve
        return max(1.0 - normalized_distance, 0.0);
    }
}

fn get_fog_density(distance: f32) -> f32 {
    // Classic fog rendering
    let fog_start = uniforms.fog_start;
    let fog_end = uniforms.fog_end;
    
    if distance < fog_start {
        return 0.0;
    } else if distance > fog_end {
        return 1.0;
    } else {
        return (distance - fog_start) / (fog_end - fog_start);
    }
}

// Additional utility functions

fn hash2d(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash2d(i);
    let b = hash2d(i + vec2<f32>(1.0, 0.0));
    let c = hash2d(i + vec2<f32>(0.0, 1.0));
    let d = hash2d(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn fbm(p: vec2<f32>) -> f32 {
    let value = 0.0;
    let amplitude = 0.5;
    let frequency = 0.0;
    
    for i in 0..4 {
        value += amplitude * noise2d(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

// Classic Minecraft texture generation functions

fn generate_programmer_art_noise(uv: vec2<f32>, texture_type: u32) -> vec4<f32> {
    let base_color = get_base_color(texture_type);
    let noise = fbm(uv * 4.0);
    
    // Apply noise with high contrast
    let noise_value = (noise - 0.5) * 2.0; // Normalize to -1.0 to 1.0
    let noise_intensity = abs(noise_value) * 0.3; // 30% noise max
    
    // Apply contrast
    let contrast_factor = 1.2;
    let r = (base_color.r * contrast_factor).clamp(0.0, 1.0);
    let g = (base_color.g * contrast_factor).clamp(0.0, 1.0);
    let b = (base_color.b * contrast_factor).clamp(0.0, 1.0);
    
    // Apply saturation
    let saturation_factor = 0.8;
    let max_rgb = max(r, max(g, b));
    let min_rgb = min(r, min(g, b));
    let gray_level = (r + g + b) / 3.0;
    
    let saturation_boost = saturation_factor * (max_rgb - gray_level);
    let r = clamp(r + saturation_boost, 0.0, 1.0);
    let g = clamp(g + saturation_boost, 0.0, 1.0);
    let b = clamp(b + saturation_boost, 0.0, 1.0);
    
    // Apply noise intensity
    let r = clamp(r + noise_intensity * 0.1, 0.0, 1.0);
    let g = clamp(g + noise_intensity * 0.1, 0.0, 1.0);
    let b = clamp(b + noise_intensity * 0.1, 0.0, 1.0);
    
    return vec4<f32>(r, g, b, 1.0);
}

fn get_base_color(texture_type: u32) -> vec4<f32> {
    // Classic Minecraft programmer art colors
    switch texture_type {
        case 0: return vec4<f32>(139.0/255.0, 90.0/255.0, 69.0/255.0, 1.0); // dirt
        case 1: return vec4<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0, 1.0); // stone
        case 2: return vec4<f32>(124.0/255.0, 169.0/255.0, 80.0/255.0, 1.0); // grass
        case 3: return vec4<f32>(238.0/255.0, 220.0/255.0, 194.0/255.0, 1.0); // sand
        case 4: return vec4<f32>(143.0/255.0, 101.0/255.0, 69.0/255.0, 1.0); // wood
        case 5: return vec4<f32>(34.0/255.0, 89.0/255.0, 34.0/255.0, 1.0); // leaves
        case 6: return vec4<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0, 1.0); // cobblestone
        case 7: return vec4<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0, 1.0); // gravel
        case 8: return vec4<f32>(24.0/255.0, 24.0/255.0, 24.0/255.0, 1.0); // coal
        case 9: return vec4<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0, 1.0); // iron_ore
        case 10: return vec4<f32>(255.0/255.0, 215.0/255.0, 0.0/255.0, 1.0); // gold_ore
        case 11: return vec4<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0, 1.0); // diamond_ore
        default: return vec4<f32>(128.0/255.0, 128.0/255.0, 128.0/255.0, 1.0); // default
    }
}
