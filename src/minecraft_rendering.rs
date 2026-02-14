//! DIABOLICAL MINECRAFT RENDERING SYSTEM - Enhanced Traditional Rendering
//! 
//! This module provides comprehensive Minecraft rendering with:
//! - Traditional visual enhancement integration
//! - Enhanced lighting and shading systems
//! - Material-specific rendering properties
//! - Biome and time-based effects
//! - Classic Minecraft aesthetic with modern performance

use crate::traditional_textures::TraditionalTextureAtlas;
use crate::config_system::GameConfig;
use wgpu::util::DeviceExt;
use glam::{Vec2, Vec3};
use std::sync::Arc;
use std::collections::HashMap;

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
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: Arc<GameConfig>,
    
    // Traditional texture system
    traditional_atlas: Arc<TraditionalTextureAtlas>,
    traditional_texture_bind_group: Option<wgpu::BindGroup>,
    material_properties_bind_group: Option<wgpu::BindGroup>,
    
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
    
    // Shader pipelines
    traditional_pipeline: Option<wgpu::RenderPipeline>,
    enhanced_pipeline: Option<wgpu::RenderPipeline>,
    classic_pipeline: Option<wgpu::RenderPipeline>,
    
    // Uniform buffers
    uniform_buffer: Option<wgpu::Buffer>,
    material_buffer: Option<wgpu::Buffer>,
    
    // Bind group layouts
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl MinecraftRenderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, config: Arc<GameConfig>) -> Self {
        let mut traditional_atlas = TraditionalTextureAtlas::new();
        traditional_atlas.generate_all_traditional_textures();
        
        Self {
            device,
            queue,
            config,
            traditional_atlas: Arc::new(traditional_atlas),
            traditional_texture_bind_group: None,
            material_properties_bind_group: None,
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
            traditional_pipeline: None,
            enhanced_pipeline: None,
            classic_pipeline: None,
            uniform_buffer: None,
            material_buffer: None,
            bind_group_layout: None,
            texture_bind_group_layout: None,
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

    pub fn calculate_vertex_light(&self, light_level: u8, vertex_pos: Vec3, surrounding_lights: [u8; 4]) -> f32 {
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

    pub fn render_sky_with_classic_fog(&self, sky_color: [f32; 4], player_pos: Vec3, render_distance: f32) -> [f32; 4] {
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

pub struct ClassicBlockRenderer {
    pub minecraft_renderer: MinecraftRenderer,
    pub texture_generator: TextureGenerator,
    pub texture_atlas: HashMap<String, Vec<u8>>,
}

impl ClassicBlockRenderer {
    pub fn new() -> Self {
        Self {
            minecraft_renderer: MinecraftRenderer::new(),
            texture_generator: TextureGenerator::new(),
            texture_atlas: HashMap::new(),
        }
    }

    pub fn initialize_textures(&mut self) {
        // Generate classic programmer art style textures
        let texture_types = vec![
            "dirt", "stone", "grass", "sand", "wood", "leaves", 
            "cobblestone", "gravel", "coal", "iron_ore", "gold_ore", "diamond_ore"
        ];

        for texture_type in texture_types {
            let texture_data = self.texture_generator.generate_programmer_art_texture(texture_type, 16);
            self.texture_atlas.insert(texture_type.to_string(), texture_data);
        }
    }

    pub fn get_texture(&self, texture_type: &str) -> Option<&Vec<u8>> {
        self.texture_atlas.get(texture_type)
    }

    pub fn render_block(
        &self,
        block_type: &str,
        world_pos: Vec3,
        normal: Vec3,
        light_level: u8,
        surrounding_lights: [u8; 4],
        distance: f32,
        uv: Vec2,
        is_transparent: bool,
    ) -> [f32; 4] {
        let base_color = if let Some(texture) = self.get_texture(block_type) {
            // Get pixel from texture atlas
            let x = (uv.x * 16.0) as usize;
            let y = (uv.y * 16.0) as usize;
            let index = (y * 16 + x) * 4;
            
            if index < texture.len() {
                [
                    texture[index] as f32 / 255.0,
                    texture[index + 1] as f32 / 255.0,
                    texture[index + 2] as f32 / 255.0,
                    texture[index + 3] as f32 / 255.0,
                ]
            } else {
                [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0]
            }
        } else {
            // Fallback colors
            match block_type {
                "dirt" => [139.0 / 255.0, 90.0 / 255.0, 69.0 / 255.0, 62.0 / 255.0],
                "stone" => [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0],
                "grass" => [124.0 / 255.0, 169.0 / 255.0, 80.0 / 255.0, 62.0 / 255.0],
                "sand" => [238.0 / 255.0, 220.0 / 255.0, 194.0 / 255.0, 174.0 / 255.0],
                "wood" => [143.0 / 255.0, 101.0 / 255.0, 69.0 / 255.0, 62.0 / 255.0],
                "leaves" => [34.0 / 255.0, 89.0 / 255.0, 34.0 / 255.0, 89.0 / 255.0],
                "cobblestone" => [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0],
                "gravel" => [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0],
                "coal" => [24.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0],
                "iron_ore" => [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0],
                "gold_ore" => [255.0 / 255.0, 215.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0],
                "diamond_ore" => [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0],
                _ => [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0],
            }
        };

        self.minecraft_renderer.render_block_with_classic_lighting(
            base_color,
            world_pos,
            normal,
            light_level,
            surrounding_lights,
            distance,
            is_transparent,
        )
    }

    pub fn render_sky(&self, sky_color: [f32; 4], player_pos: Vec3, render_distance: f32) -> [f32; 4] {
        self.minecraft_renderer.render_sky_with_classic_fog(sky_color, player_pos, render_distance)
    }

    pub fn apply_pillow_shading_to_uv(&self, normal: Vec3, uv: Vec2) -> Vec2 {
        self.minecraft_renderer.apply_pillow_shading(normal, uv)
    }

    pub fn get_view_bobbing(&self, time: f32) -> Vec3 {
        self.minecraft_renderer.apply_view_bobbing(time)
    }

    pub fn get_fog_density(&self, distance: f32) -> f32 {
        self.minecraft_renderer.get_fog_density(distance)
    }

    pub fn get_texture_filter_mode(&self) -> u32 {
        self.minecraft_renderer.get_texture_filter_mode()
    }

    pub fn should_use_mipmapping(&self) -> bool {
        self.minecraft_renderer.should_use_mipmapping()
    }

    pub fn set_rendering_mode(&mut self, mode: ShadingMode) {
        self.minecraft_renderer.set_shading_mode(mode);
    }

    pub fn set_fog_settings(&mut self, fog_type: FogType, start: f32, end: f32, color: [f32; 4]) {
        self.minecraft_renderer.set_fog_type(fog_type);
        self.minecraft_renderer.fog_start = start;
        self.minecraft_renderer.fog_end = end;
        self.minecraft_renderer.fog_color = color;
    }
}
