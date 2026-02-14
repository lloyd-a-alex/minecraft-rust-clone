// Traditional Minecraft Enhanced Shaders
// Implements hand-crafted texture rendering with material properties
// and advanced lighting while maintaining classic Minecraft aesthetic

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) light_level: u32,
    @location(5) surrounding_lights: vec4<u32>,
    @location(6) world_pos: vec3<f32>,
    @location(7) material_type: u32,
    @location(8) biome_modifier: u32,
    @location(9) time_modifier: u32,
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
    @location(7) frag_material_type: u32,
    @location(8) frag_biome_modifier: u32,
    @location(9) frag_time_modifier: u32,
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
    @group(0) @binding(12) traditional_rendering: f32,
    @group(0) @binding(13) material_properties: f32,
    @group(0) @binding(14) biome_time_effects: f32,
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
    
    // Pass through all data
    output.frag_color = input.color;
    output.frag_uv = input.uv;
    output.frag_normal = input.normal;
    output.frag_light_level = input.light_level;
    output.frag_surrounding_lights = input.surrounding_lights;
    output.frag_material_type = input.material_type;
    output.frag_biome_modifier = input.biome_modifier;
    output.frag_time_modifier = input.time_modifier;
    
    return output;
}

@group(1) @binding(0)
var texture_sampler: sampler;

@group(1) @binding(1)
var traditional_atlas: texture_2d<f32>;

@group(1) @binding(2)
var material_properties: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample traditional texture with nearest-neighbor filtering
    let tex_color = textureSample(traditional_atlas, texture_sampler, input.frag_uv);
    
    // Get material properties
    let material_uv = vec2<f32>(
        (input.frag_material_type as f32 + 0.5) / 16.0,
        0.5
    );
    let material_props = textureSample(material_properties, texture_sampler, material_uv);
    
    // Apply traditional rendering if enabled
    var final_color = tex_color;
    
    if uniforms.traditional_rendering > 0.5 {
        // Apply material-specific lighting
        final_color = apply_traditional_lighting(
            final_color,
            input.frag_world_pos,
            input.frag_normal,
            input.frag_light_level,
            input.frag_surrounding_lights,
            input.frag_distance,
            material_props,
            input.frag_material_type
        );
        
        // Apply biome and time effects
        if uniforms.biome_time_effects > 0.5 {
            final_color = apply_biome_time_effects(
                final_color,
                input.frag_biome_modifier,
                input.frag_time_modifier,
                uniforms.time
            );
        }
    } else {
        // Fall back to classic rendering
        final_color = apply_classic_lighting(
            final_color,
            input.frag_world_pos,
            input.frag_normal,
            input.frag_light_level,
            input.frag_surrounding_lights,
            input.frag_distance
        );
    }
    
    // Apply fog
    let fog_density = get_fog_density(input.frag_distance);
    let fog_color = uniforms.fog_color;
    
    // Linear interpolation between block color and fog color
    final_color = mix(final_color, fog_color, fog_density);
    
    return final_color;
}

fn apply_traditional_lighting(
    base_color: vec4<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    light_level: u32,
    surrounding_lights: vec4<u32>,
    distance: f32,
    material_props: vec4<f32>,
    material_type: u32
) -> vec4<f32> {
    var final_color = base_color;
    
    // Extract material properties
    let roughness = material_props.r;
    let metallic = material_props.g;
    let transparency = material_props.b;
    let emission = material_props.a;
    
    // Apply directional face shading
    if uniforms.directional_shading > 0.5 {
        let directional_multiplier = get_directional_multiplier(normal);
        final_color = final_color * directional_multiplier;
    }
    
    // Apply vertex ambient occlusion
    if uniforms.ambient_occlusion > 0.5 {
        let vertex_light = calculate_vertex_light(light_level, world_pos, surrounding_lights);
        final_color = final_color * vertex_light;
    }
    
    // Apply material-specific lighting
    if uniforms.material_properties > 0.5 {
        final_color = apply_material_lighting(final_color, material_props, normal, roughness, metallic);
    }
    
    // Apply logarithmic light attenuation
    if uniforms.logarithmic_lighting > 0.5 {
        let attenuation = apply_logarithmic_attenuation(distance);
        final_color = final_color * attenuation;
    }
    
    // Apply emission
    if emission > 0.0 {
        final_color = final_color + vec4<f32>(emission, emission, emission, 0.0);
    }
    
    // Apply transparency
    if transparency > 0.0 {
        final_color.a = final_color.a * (1.0 - transparency);
    }
    
    return final_color;
}

fn apply_material_lighting(
    color: vec4<f32>,
    material_props: vec4<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metallic: f32
) -> vec4<f32> {
    var final_color = color;
    
    // Apply roughness-based lighting
    let roughness_factor = 1.0 - roughness * 0.3;
    final_color.rgb = final_color.rgb * roughness_factor;
    
    // Apply metallic highlights
    if metallic > 0.5 {
        let view_dir = normalize(uniforms.camera_pos - vec3<f32>(0.0, 0.0, 0.0));
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let half_dir = normalize(view_dir + light_dir);
        let specular = pow(max(dot(normal, half_dir), 0.0), 32.0) * metallic;
        
        final_color.rgb = final_color.rgb + vec3<f32>(specular * 0.3);
    }
    
    return final_color;
}

fn apply_biome_time_effects(
    color: vec4<f32>,
    biome_modifier: u32,
    time_modifier: u32,
    current_time: f32
) -> vec4<f32> {
    var final_color = color;
    
    // Apply biome effects
    if biome_modifier > 0 {
        final_color = apply_biome_modifier(final_color, biome_modifier);
    }
    
    // Apply time effects
    if time_modifier > 0 {
        final_color = apply_time_modifier(final_color, time_modifier, current_time);
    }
    
    return final_color;
}

fn apply_biome_modifier(color: vec4<f32>, biome_modifier: u32) -> vec4<f32> {
    var final_color = color;
    
    switch biome_modifier {
        case 1: // Forest
            final_color.r = final_color.r * 0.9;
            final_color.g = final_color.g * 1.2;
            final_color.b = final_color.b * 0.8;
        case 2: // Desert
            final_color.r = final_color.r * 1.3;
            final_color.g = final_color.g * 1.1;
            final_color.b = final_color.b * 0.7;
        case 3: // Tundra
            final_color.r = final_color.r * 0.8;
            final_color.g = final_color.g * 0.9;
            final_color.b = final_color.b * 1.2;
        case 4: // Swamp
            final_color.r = final_color.r * 0.9;
            final_color.g = final_color.g * 1.1;
            final_color.b = final_color.b * 0.8;
        default:
            // No modification
    }
    
    return final_color;
}

fn apply_time_modifier(color: vec4<f32>, time_modifier: u32, current_time: f32) -> vec4<f32> {
    var final_color = color;
    
    // Apply time-based color shifts
    let time_factor = sin(current_time * 0.1) * 0.5 + 0.5;
    
    switch time_modifier {
        case 1: // Dawn
            final_color.r = final_color.r * (1.0 + time_factor * 0.2);
            final_color.g = final_color.g * (1.0 + time_factor * 0.1);
            final_color.b = final_color.b * (1.0 - time_factor * 0.1);
        case 2: // Noon
            final_color = final_color * (1.0 + time_factor * 0.1);
        case 3: // Dusk
            final_color.r = final_color.r * (1.0 + time_factor * 0.3);
            final_color.g = final_color.g * (1.0 - time_factor * 0.1);
            final_color.b = final_color.b * (1.0 - time_factor * 0.3);
        case 4: // Night
            final_color.r = final_color.r * (1.0 - time_factor * 0.3);
            final_color.g = final_color.g * (1.0 - time_factor * 0.2);
            final_color.b = final_color.b * (1.0 + time_factor * 0.2);
        default:
            // No modification
    }
    
    return final_color;
}

fn apply_classic_lighting(
    base_color: vec4<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    light_level: u32,
    surrounding_lights: vec4<u32>,
    distance: f32
) -> vec4<f32> {
    var final_color = base_color;
    
    // Apply directional face shading
    if uniforms.directional_shading > 0.5 {
        let directional_multiplier = get_directional_multiplier(normal);
        final_color = final_color * directional_multiplier;
    }
    
    // Apply vertex ambient occlusion
    if uniforms.ambient_occlusion > 0.5 {
        let vertex_light = calculate_vertex_light(light_level, world_pos, surrounding_lights);
        final_color = final_color * vertex_light;
    }
    
    // Apply logarithmic light attenuation
    if uniforms.logarithmic_lighting > 0.5 {
        let attenuation = apply_logarithmic_attenuation(distance);
        final_color = final_color * attenuation;
    }
    
    return final_color;
}

fn get_directional_multiplier(normal: vec3<f32>) -> f32 {
    // Classic Minecraft directional shading multipliers
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

// Utility functions for enhanced rendering

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

// Enhanced texture generation functions for traditional rendering

fn generate_traditional_programmer_art(uv: vec2<f32>, texture_type: u32) -> vec4<f32> {
    let base_color = get_traditional_base_color(texture_type);
    let noise = fbm(uv * 4.0);
    
    // Apply traditional noise with high contrast
    let noise_value = (noise - 0.5) * 2.0;
    let noise_intensity = abs(noise_value) * 0.3;
    
    // Apply traditional contrast and saturation
    let contrast_factor = 1.2;
    let saturation_factor = 0.8;
    
    let r = (base_color.r * contrast_factor).clamp(0.0, 1.0);
    let g = (base_color.g * contrast_factor).clamp(0.0, 1.0);
    let b = (base_color.b * contrast_factor).clamp(0.0, 1.0);
    
    // Apply traditional saturation
    let max_rgb = max(r, max(g, b));
    let gray_level = (r + g + b) / 3.0;
    
    if gray_level > 0.1 {
        let saturation_boost = saturation_factor * (max_rgb - gray_level);
        r = clamp(r + saturation_boost, 0.0, 1.0);
        g = clamp(g + saturation_boost, 0.0, 1.0);
        b = clamp(b + saturation_boost, 0.0, 1.0);
    }
    
    // Apply traditional noise intensity
    r = clamp(r + noise_intensity * 0.1, 0.0, 1.0);
    g = clamp(g + noise_intensity * 0.1, 0.0, 1.0);
    b = clamp(b + noise_intensity * 0.1, 0.0, 1.0);
    
    return vec4<f32>(r, g, b, 1.0);
}

fn get_traditional_base_color(texture_type: u32) -> vec3<f32> {
    // Traditional Minecraft programmer art colors with enhanced palette
    switch texture_type {
        case 0: return vec3<f32>(139.0/255.0, 90.0/255.0, 69.0/255.0); // Enhanced dirt
        case 1: return vec3<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0); // Enhanced stone
        case 2: return vec3<f32>(124.0/255.0, 169.0/255.0, 80.0/255.0); // Enhanced grass
        case 3: return vec3<f32>(238.0/255.0, 220.0/255.0, 194.0/255.0); // Enhanced sand
        case 4: return vec3<f32>(143.0/255.0, 101.0/255.0, 69.0/255.0); // Enhanced wood
        case 5: return vec3<f32>(34.0/255.0, 89.0/255.0, 34.0/255.0); // Enhanced leaves
        case 6: return vec3<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0); // Enhanced cobblestone
        case 7: return vec3<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0); // Enhanced gravel
        case 8: return vec3<f32>(24.0/255.0, 24.0/255.0, 24.0/255.0); // Enhanced coal
        case 9: return vec3<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0); // Enhanced iron ore
        case 10: return vec3<f32>(255.0/255.0, 215.0/255.0, 0.0/255.0); // Enhanced gold ore
        case 11: return vec3<f32>(136.0/255.0, 136.0/255.0, 136.0/255.0); // Enhanced diamond ore
        case 12: return vec3<f32>(80.0/255.0, 120.0/255.0, 180.0/255.0); // Enhanced water
        case 13: return vec3<f32>(200.0/255.0, 200.0/255.0, 200.0/255.0); // Enhanced glass
        case 14: return vec3<f32>(192.0/255.0, 192.0/255.0, 192.0/255.0); // Enhanced metal
        case 15: return vec3<f32>(200.0/255.0, 200.0/255.0, 255.0/255.0); // Enhanced crystal
        default: return vec3<f32>(128.0/255.0, 128.0/255.0, 128.0/255.0); // Default
    }
}
