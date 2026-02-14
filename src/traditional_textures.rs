//! DIABOLICAL TRADITIONAL TEXTURE SYSTEM - Hand-Crafted Visual Enhancement
//! 
//! This module provides comprehensive traditional texture generation with:
//! - Artist-designed templates for authentic Minecraft aesthetics
//! - Layered texture composition for depth and detail
//! - Traditional color palettes inspired by classic Minecraft
//! - Material-specific rendering properties
//! - Biome and time-based texture variations

use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Stone,
    Wood,
    Dirt,
    Grass,
    Sand,
    Water,
    Leaves,
    Metal,
    Glass,
    Fabric,
    Crystal,
}

#[derive(Debug, Clone)]
pub struct TextureTemplate {
    pub base_color: [u8; 3],
    pub detail_colors: Vec<[u8; 3]>,
    pub material_type: MaterialType,
    pub pattern_type: PatternType,
    pub roughness: f32,
    pub metallic: f32,
    pub transparency: f32,
    pub emission: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternType {
    Solid,
    Grain,
    Veined,
    Crystalline,
    Fabric,
    Metallic,
    Organic,
    Geometric,
}

#[derive(Debug, Clone)]
pub struct TextureLayer {
    pub pattern: PatternType,
    pub color: [u8; 3],
    pub intensity: f32,
    pub scale: f32,
    pub offset: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct TraditionalPalette {
    pub stone_colors: [[u8; 3]; 8],
    pub wood_colors: [[u8; 3]; 6],
    pub dirt_colors: [[u8; 3]; 4],
    pub grass_colors: [[u8; 3]; 5],
    pub sand_colors: [[u8; 3]; 3],
    pub ore_colors: [[u8; 3]; 10],
    pub plant_colors: [[u8; 3]; 12],
    pub metal_colors: [[u8; 3]; 8],
}

pub struct TraditionalTextureGenerator {
    pub palette: TraditionalPalette,
    pub templates: HashMap<MaterialType, TextureTemplate>,
    pub noise_scale: f32,
    pub detail_level: f32,
}

impl TraditionalTextureGenerator {
    pub fn new() -> Self {
        let palette = Self::create_traditional_palette();
        let templates = Self::create_texture_templates(&palette);
        
        Self {
            palette,
            templates,
            noise_scale: 0.1,
            detail_level: 0.8,
        }
    }

    fn create_traditional_palette() -> TraditionalPalette {
        TraditionalPalette {
            // Warm, traditional stone colors
            stone_colors: [
                [136, 136, 136], // Light stone
                [119, 119, 119], // Medium stone
                [102, 102, 102], // Dark stone
                [85, 85, 85],     // Very dark stone
                [153, 153, 153], // Pale stone
                [170, 170, 170], // Bright stone
                [68, 68, 68],     // Shadow stone
                [187, 187, 187], // Weathered stone
            ],
            // Natural wood colors with grain
            wood_colors: [
                [143, 101, 69],  // Oak wood
                [92, 51, 23],    // Dark oak
                [194, 178, 128], // Birch wood
                [113, 67, 25],   // Spruce wood
                [160, 106, 66],  // Jungle wood
                [247, 233, 163], // Acacia wood
            ],
            // Earth tones
            dirt_colors: [
                [139, 90, 69],   // Light dirt
                [121, 85, 61],   // Medium dirt
                [109, 77, 54],   // Dark dirt
                [155, 118, 87],  // Sandy dirt
            ],
            // Natural grass colors
            grass_colors: [
                [124, 169, 80],  // Healthy grass
                [134, 179, 90],  // Lush grass
                [114, 159, 70],  // Dry grass
                [144, 189, 100], // Tropical grass
                [104, 149, 60],  // Sparse grass
            ],
            // Sandy colors
            sand_colors: [
                [238, 220, 194], // Light sand
                [218, 200, 174], // Medium sand
                [198, 180, 154], // Dark sand
            ],
            // Rich ore colors
            ore_colors: [
                [24, 24, 24],     // Coal
                [210, 180, 140], // Iron
                [255, 215, 0],   // Gold
                [0, 255, 255],    // Diamond
                [255, 0, 0],     // Redstone
                [20, 40, 180],   // Lapis
                [160, 160, 160], // Silver
                [255, 128, 0],   // Copper
                [128, 0, 128],   // Amethyst
                [255, 255, 255], // Quartz
            ],
            // Vibrant plant colors
            plant_colors: [
                [34, 89, 34],    // Green leaves
                [124, 169, 80],  // Grass green
                [255, 255, 0],   // Yellow flowers
                [255, 0, 0],     // Red flowers
                [128, 0, 128],   // Purple flowers
                [255, 192, 203], // Pink flowers
                [255, 165, 0],   // Orange flowers
                [0, 0, 255],     // Blue flowers
                [255, 255, 255], // White flowers
                [165, 42, 42],   // Brown mushrooms
                [255, 0, 255],   // Magenta flowers
                [192, 192, 192], // Gray flowers
            ],
            // Metallic colors
            metal_colors: [
                [192, 192, 192], // Iron
                [255, 215, 0],   // Gold
                [192, 192, 192], // Steel
                [184, 115, 51],  // Copper
                [128, 128, 128], // Lead
                [255, 255, 255], // Silver
                [217, 217, 217], // Aluminum
                [255, 140, 0],   // Bronze
            ],
        }
    }

    fn create_texture_templates(palette: &TraditionalPalette) -> HashMap<MaterialType, TextureTemplate> {
        let mut templates = HashMap::new();

        // Stone template with grain pattern
        templates.insert(MaterialType::Stone, TextureTemplate {
            base_color: palette.stone_colors[1],
            detail_colors: palette.stone_colors[1..].to_vec(),
            material_type: MaterialType::Stone,
            pattern_type: PatternType::Grain,
            roughness: 0.8,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Wood template with visible grain
        templates.insert(MaterialType::Wood, TextureTemplate {
            base_color: palette.wood_colors[0],
            detail_colors: palette.wood_colors[1..].to_vec(),
            material_type: MaterialType::Wood,
            pattern_type: PatternType::Grain,
            roughness: 0.6,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Dirt template with organic pattern
        templates.insert(MaterialType::Dirt, TextureTemplate {
            base_color: palette.dirt_colors[0],
            detail_colors: palette.dirt_colors[1..].to_vec(),
            material_type: MaterialType::Dirt,
            pattern_type: PatternType::Organic,
            roughness: 0.9,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Grass template with organic pattern
        templates.insert(MaterialType::Grass, TextureTemplate {
            base_color: palette.grass_colors[0],
            detail_colors: palette.grass_colors[1..].to_vec(),
            material_type: MaterialType::Grass,
            pattern_type: PatternType::Organic,
            roughness: 0.7,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Sand template with fine grain
        templates.insert(MaterialType::Sand, TextureTemplate {
            base_color: palette.sand_colors[0],
            detail_colors: palette.sand_colors[1..].to_vec(),
            material_type: MaterialType::Sand,
            pattern_type: PatternType::Grain,
            roughness: 0.5,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Water template with transparency
        templates.insert(MaterialType::Water, TextureTemplate {
            base_color: [80, 120, 180],
            detail_colors: vec![[60, 100, 160], [100, 140, 200]],
            material_type: MaterialType::Water,
            pattern_type: PatternType::Solid,
            roughness: 0.1,
            metallic: 0.0,
            transparency: 0.7,
            emission: 0.0,
        });

        // Leaves template with veined pattern
        templates.insert(MaterialType::Leaves, TextureTemplate {
            base_color: palette.plant_colors[0],
            detail_colors: palette.plant_colors[1..4].to_vec(),
            material_type: MaterialType::Leaves,
            pattern_type: PatternType::Veined,
            roughness: 0.4,
            metallic: 0.0,
            transparency: 0.3,
            emission: 0.0,
        });

        // Metal template with metallic properties
        templates.insert(MaterialType::Metal, TextureTemplate {
            base_color: palette.metal_colors[0],
            detail_colors: palette.metal_colors[1..].to_vec(),
            material_type: MaterialType::Metal,
            pattern_type: PatternType::Metallic,
            roughness: 0.2,
            metallic: 0.8,
            transparency: 0.0,
            emission: 0.0,
        });

        // Glass template with transparency
        templates.insert(MaterialType::Glass, TextureTemplate {
            base_color: [200, 200, 200],
            detail_colors: vec![[180, 180, 180], [220, 220, 220]],
            material_type: MaterialType::Glass,
            pattern_type: PatternType::Solid,
            roughness: 0.0,
            metallic: 0.0,
            transparency: 0.8,
            emission: 0.0,
        });

        // Fabric template
        templates.insert(MaterialType::Fabric, TextureTemplate {
            base_color: [200, 150, 100],
            detail_colors: vec![[180, 130, 80], [220, 170, 120]],
            material_type: MaterialType::Fabric,
            pattern_type: PatternType::Fabric,
            roughness: 0.6,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Crystal template with crystalline pattern
        templates.insert(MaterialType::Crystal, TextureTemplate {
            base_color: [200, 200, 255],
            detail_colors: vec![[180, 180, 235], [220, 220, 255]],
            material_type: MaterialType::Crystal,
            pattern_type: PatternType::Crystalline,
            roughness: 0.1,
            metallic: 0.2,
            transparency: 0.6,
            emission: 0.1,
        });

        templates
    }

    pub fn generate_traditional_texture(
        &self,
        material_type: MaterialType,
        size: usize,
        biome_modifier: Option<BiomeModifier>,
        time_modifier: Option<TimeModifier>,
    ) -> Vec<u8> {
        let template = &self.templates[&material_type];
        let mut texture_data = Vec::with_capacity(size * size * 4);

        for y in 0..size {
            for x in 0..size {
                let uv = [x as f32 / size as f32, y as f32 / size as f32];
                
                // Generate base color with pattern
                let base_color = self.generate_pattern_color(template, uv);
                
                // Apply biome and time modifiers
                let modified_color = self.apply_modifiers(base_color, biome_modifier, time_modifier);
                
                // Add detail layers
                let final_color = self.add_detail_layers(template, modified_color, uv);
                
                texture_data.extend([final_color[0], final_color[1], final_color[2], 255]);
            }
        }

        texture_data
    }

    fn generate_pattern_color(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        match template.pattern_type {
            PatternType::Solid => template.base_color,
            PatternType::Grain => self.generate_grain_pattern(template, uv),
            PatternType::Veined => self.generate_veined_pattern(template, uv),
            PatternType::Crystalline => self.generate_crystalline_pattern(template, uv),
            PatternType::Fabric => self.generate_fabric_pattern(template, uv),
            PatternType::Metallic => self.generate_metallic_pattern(template, uv),
            PatternType::Organic => self.generate_organic_pattern(template, uv),
            PatternType::Geometric => self.generate_geometric_pattern(template, uv),
        }
    }

    fn generate_grain_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let noise = self.perlin_noise(uv[0] * 8.0, uv[1] * 8.0);
        let color_index = (noise * template.detail_colors.len() as f32) as usize % template.detail_colors.len();
        template.detail_colors[color_index]
    }

    fn generate_veined_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let vein_noise = self.perlin_noise(uv[0] * 16.0, uv[1] * 16.0);
        let base_noise = self.perlin_noise(uv[0] * 4.0, uv[1] * 4.0);
        
        if vein_noise > 0.7 {
            template.detail_colors[0] // Vein color
        } else {
            self.blend_colors(template.base_color, template.detail_colors[1], base_noise)
        }
    }

    fn generate_crystalline_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let crystal_noise = self.perlin_noise(uv[0] * 12.0, uv[1] * 12.0);
        let facet_noise = self.perlin_noise(uv[0] * 24.0, uv[1] * 24.0);
        
        let base_color = self.blend_colors(template.base_color, template.detail_colors[0], crystal_noise);
        self.blend_colors(base_color, template.detail_colors[1], facet_noise * 0.3)
    }

    fn generate_fabric_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let weave_x = (uv[0] * 20.0).sin() * 0.5 + 0.5;
        let weave_y = (uv[1] * 20.0).cos() * 0.5 + 0.5;
        let weave_pattern = (weave_x * weave_y).powf(2.0);
        
        self.blend_colors(template.base_color, template.detail_colors[0], weave_pattern)
    }

    fn generate_metallic_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let metal_noise = self.perlin_noise(uv[0] * 32.0, uv[1] * 32.0);
        let polish_pattern = (uv[0] * 100.0).sin() * (uv[1] * 100.0).cos() * 0.1 + 0.9;
        
        let base_color = self.blend_colors(template.base_color, template.detail_colors[0], metal_noise);
        self.blend_colors(base_color, template.detail_colors[1], polish_pattern)
    }

    fn generate_organic_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let organic_noise = self.perlin_noise(uv[0] * 6.0, uv[1] * 6.0);
        let detail_noise = self.perlin_noise(uv[0] * 12.0, uv[1] * 12.0);
        
        self.blend_colors(template.base_color, template.detail_colors[0], organic_noise * 0.7 + detail_noise * 0.3)
    }

    fn generate_geometric_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let grid_x = (uv[0] * 8.0) as usize % 2;
        let grid_y = (uv[1] * 8.0) as usize % 2;
        let checker_pattern = if (grid_x + grid_y) % 2 == 0 { 1.0 } else { 0.0 };
        
        self.blend_colors(template.base_color, template.detail_colors[0], checker_pattern)
    }

    fn apply_modifiers(
        &self,
        color: [u8; 3],
        biome_modifier: Option<BiomeModifier>,
        time_modifier: Option<TimeModifier>,
    ) -> [u8; 3] {
        let mut modified_color = color;
        
        if let Some(biome) = biome_modifier {
            modified_color = self.apply_biome_modifier(modified_color, biome);
        }
        
        if let Some(time) = time_modifier {
            modified_color = self.apply_time_modifier(modified_color, time);
        }
        
        modified_color
    }

    fn apply_biome_modifier(&self, color: [u8; 3], biome: BiomeModifier) -> [u8; 3] {
        match biome {
            BiomeModifier::Forest => {
                // Greener, more vibrant colors
                [
                    (color[0] as f32 * 0.9).max(0.0) as u8,
                    (color[1] as f32 * 1.2).min(255.0) as u8,
                    (color[2] as f32 * 0.8).max(0.0) as u8,
                ]
            }
            BiomeModifier::Desert => {
                // Warmer, more orange tones
                [
                    (color[0] as f32 * 1.3).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.7).max(0.0) as u8,
                ]
            }
            BiomeModifier::Tundra => {
                // Cooler, more blue tones
                [
                    (color[0] as f32 * 0.8).max(0.0) as u8,
                    (color[1] as f32 * 0.9).max(0.0) as u8,
                    (color[2] as f32 * 1.2).min(255.0) as u8,
                ]
            }
            BiomeModifier::Swamp => {
                // Murkier, more green-brown tones
                [
                    (color[0] as f32 * 0.9).max(0.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.8).max(0.0) as u8,
                ]
            }
        }
    }

    fn apply_time_modifier(&self, color: [u8; 3], time: TimeModifier) -> [u8; 3] {
        match time {
            TimeModifier::Dawn => {
                // Warmer, more orange-pink tones
                [
                    (color[0] as f32 * 1.2).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.9).max(0.0) as u8,
                ]
            }
            TimeModifier::Noon => {
                // Brighter, more saturated
                [
                    (color[0] as f32 * 1.1).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 1.1).min(255.0) as u8,
                ]
            }
            TimeModifier::Dusk => {
                // Warmer, more orange-red tones
                [
                    (color[0] as f32 * 1.3).min(255.0) as u8,
                    (color[1] as f32 * 0.9).max(0.0) as u8,
                    (color[2] as f32 * 0.7).max(0.0) as u8,
                ]
            }
            TimeModifier::Night => {
                // Cooler, more blue tones
                [
                    (color[0] as f32 * 0.7).max(0.0) as u8,
                    (color[1] as f32 * 0.8).max(0.0) as u8,
                    (color[2] as f32 * 1.2).min(255.0) as u8,
                ]
            }
        }
    }

    fn add_detail_layers(&self, template: &TextureTemplate, base_color: [u8; 3], uv: [f32; 2]) -> [u8; 3] {
        let mut final_color = base_color;
        
        // Add noise-based detail
        let detail_noise = self.perlin_noise(uv[0] * 16.0, uv[1] * 16.0);
        let detail_intensity = detail_noise * self.detail_level;
        
        // Blend with detail color
        if detail_intensity > 0.5 {
            let detail_color = template.detail_colors[1];
            final_color = self.blend_colors(final_color, detail_color, (detail_intensity - 0.5) * 2.0);
        }
        
        // Add edge highlighting
        let edge_noise = self.perlin_noise(uv[0] * 32.0, uv[1] * 32.0);
        if edge_noise > 0.8 {
            let highlight_color = [
                (final_color[0] as f32 * 1.2).min(255.0) as u8,
                (final_color[1] as f32 * 1.2).min(255.0) as u8,
                (final_color[2] as f32 * 1.2).min(255.0) as u8,
            ];
            final_color = self.blend_colors(final_color, highlight_color, (edge_noise - 0.8) * 5.0);
        }
        
        final_color
    }

    fn blend_colors(&self, color1: [u8; 3], color2: [u8; 3], factor: f32) -> [u8; 3] {
        let factor = factor.clamp(0.0, 1.0);
        [
            (color1[0] as f32 * (1.0 - factor) + color2[0] as f32 * factor) as u8,
            (color1[1] as f32 * (1.0 - factor) + color2[1] as f32 * factor) as u8,
            (color1[2] as f32 * (1.0 - factor) + color2[2] as f32 * factor) as u8,
        ]
    }

    fn perlin_noise(&self, x: f32, y: f32) -> f32 {
        // Simple Perlin noise implementation
        let x = x * self.noise_scale;
        let y = y * self.noise_scale;
        
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;
        
        let u = self.fade(xf);
        let v = self.fade(yf);
        
        let a = self.hash(xi, yi);
        let b = self.hash(xi + 1, yi);
        let c = self.hash(xi, yi + 1);
        let d = self.hash(xi + 1, yi + 1);
        
        let x1 = self.lerp(self.grad(a, xf, yf), self.grad(b, xf - 1.0, yf), u);
        let x2 = self.lerp(self.grad(c, xf, yf - 1.0), self.grad(d, xf - 1.0, yf - 1.0), u);
        
        self.lerp(x1, x2, v)
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    fn grad(&self, hash: i32, x: f32, y: f32) -> f32 {
        let h = hash & 3;
        let u = if h < 2 { x } else { y };
        let v = if h < 2 { y } else { x };
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }

    fn hash(&self, x: i32, y: i32) -> i32 {
        let mut h = x;
        h ^= y << 13;
        h ^= h >> 17;
        h.wrapping_mul(0x85ebca6b).wrapping_add(0xc2b2ae35)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BiomeModifier {
    Forest,
    Desert,
    Tundra,
    Swamp,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeModifier {
    Dawn,
    Noon,
    Dusk,
    Night,
}

pub struct TraditionalTextureAtlas {
    pub generator: TraditionalTextureGenerator,
    pub textures: HashMap<String, Vec<u8>>,
    pub size: u32,
    pub grid_size: u32,
}

impl TraditionalTextureAtlas {
    pub fn new() -> Self {
        let generator = TraditionalTextureGenerator::new();
        let textures = HashMap::new();
        
        Self {
            generator,
            textures,
            size: 512,
            grid_size: 32,
        }
    }

    pub fn generate_all_traditional_textures(&mut self) {
        let block_size = 16;
        
        // Natural materials
        self.generate_texture("grass", MaterialType::Grass, block_size);
        self.generate_texture("dirt", MaterialType::Dirt, block_size);
        self.generate_texture("stone", MaterialType::Stone, block_size);
        self.generate_texture("sand", MaterialType::Sand, block_size);
        self.generate_texture("water", MaterialType::Water, block_size);
        self.generate_texture("leaves", MaterialType::Leaves, block_size);
        
        // Building materials
        self.generate_texture("wood", MaterialType::Wood, block_size);
        self.generate_texture("glass", MaterialType::Glass, block_size);
        self.generate_texture("fabric", MaterialType::Fabric, block_size);
        
        // Special materials
        self.generate_texture("metal", MaterialType::Metal, block_size);
        self.generate_texture("crystal", MaterialType::Crystal, block_size);
        
        // Generate biome variants
        self.generate_biome_variants();
        
        // Generate time variants
        self.generate_time_variants();
    }

    fn generate_texture(&mut self, name: &str, material_type: MaterialType, block_size: usize) {
        let texture_data = self.generator.generate_traditional_texture(
            material_type,
            block_size,
            None,
            None,
        );
        self.textures.insert(name.to_string(), texture_data);
    }

    fn generate_biome_variants(&mut self) {
        let block_size = 16;
        let biomes = [BiomeModifier::Forest, BiomeModifier::Desert, BiomeModifier::Tundra, BiomeModifier::Swamp];
        
        for biome in &biomes {
            let suffix = match biome {
                BiomeModifier::Forest => "_forest",
                BiomeModifier::Desert => "_desert",
                BiomeModifier::Tundra => "_tundra",
                BiomeModifier::Swamp => "_swamp",
            };
            
            // Generate grass variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Grass,
                block_size,
                Some(*biome),
                None,
            );
            self.textures.insert(format!("grass{}", suffix), texture_data);
            
            // Generate leaves variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Leaves,
                block_size,
                Some(*biome),
                None,
            );
            self.textures.insert(format!("leaves{}", suffix), texture_data);
        }
    }

    fn generate_time_variants(&mut self) {
        let block_size = 16;
        let times = [TimeModifier::Dawn, TimeModifier::Noon, TimeModifier::Dusk, TimeModifier::Night];
        
        for time in &times {
            let suffix = match time {
                TimeModifier::Dawn => "_dawn",
                TimeModifier::Noon => "_noon",
                TimeModifier::Dusk => "_dusk",
                TimeModifier::Night => "_night",
            };
            
            // Generate grass time variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Grass,
                block_size,
                None,
                Some(*time),
            );
            self.textures.insert(format!("grass{}", suffix), texture_data);
            
            // Generate leaves time variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Leaves,
                block_size,
                None,
                Some(*time),
            );
            self.textures.insert(format!("leaves{}", suffix), texture_data);
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Vec<u8>> {
        self.textures.get(name)
    }

    pub fn create_atlas_data(&self) -> Vec<u8> {
        let mut atlas_data = vec![0u8; (self.size * self.size * 4) as usize];
        
        for (name, texture) in &self.textures {
            // Place texture in atlas
            let index = self.get_texture_index(name);
            if let Some((x, y)) = index {
                self.copy_texture_to_atlas(&mut atlas_data, texture, x, y);
            }
        }
        
        atlas_data
    }

    fn get_texture_index(&self, name: &str) -> Option<(u32, u32)> {
        // Simple grid placement - in a real implementation, this would be more sophisticated
        let texture_names: Vec<&String> = self.textures.keys().collect();
        if let Some(pos) = texture_names.iter().position(|&n| n == name) {
            let x = (pos as u32 % self.grid_size) * 16;
            let y = (pos as u32 / self.grid_size) * 16;
            Some((x, y))
        } else {
            None
        }
    }

    fn copy_texture_to_atlas(&self, atlas_data: &mut [u8], texture: &[u8], x: u32, y: u32) {
        let texture_size = 16;
        let atlas_width = self.size;
        
        for ty in 0..texture_size {
            for tx in 0..texture_size {
                let atlas_x = x + tx;
                let atlas_y = y + ty;
                
                if atlas_x < self.size && atlas_y < self.size {
                    let texture_index = ((ty * texture_size + tx) * 4) as usize;
                    let atlas_index = ((atlas_y * atlas_width + atlas_x) * 4) as usize;
                    
                    if atlas_index + 3 < atlas_data.len() && texture_index + 3 < texture.len() {
                        atlas_data[atlas_index..atlas_index + 4].copy_from_slice(&texture[texture_index..texture_index + 4]);
                    }
                }
            }
        }
    }
}
