use crate::world::{BlockPos, BlockType, World, Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, WORLD_HEIGHT};
use crate::noise_gen::NoiseGenerator;
use std::collections::HashMap;
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaterType {
    River,
    Ocean,
    Lake,
}

#[derive(Debug, Clone)]
pub struct RiverData {
    pub path: Vec<Vec3>,
    pub width: f32,
    pub depth: f32,
    pub flow_direction: Vec3,
    pub water_type: WaterType,
}

#[derive(Debug, Clone)]
pub struct OceanData {
    pub seed: u32,
    pub islands: Vec<Vec3>,
    pub ocean_level: f32,
    pub beach_width: f32,
}

pub struct WaterSystem {
    pub rivers: HashMap<(i32, i32, i32), RiverData>,
    pub oceans: OceanData,
    pub water_level: f32,
}

impl WaterSystem {
    pub fn new(seed: u32) -> Self {
        Self {
            rivers: HashMap::new(),
            oceans: OceanData {
                seed,
                islands: Vec::new(),
                ocean_level: 60.0,
                beach_width: 8.0,
            },
            water_level: 60.0,
        }
    }

    pub fn generate_rivers(&mut self, world: &mut World, noise_gen: &NoiseGenerator) {
        // Generate river paths using noise
        let river_count = 3 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 4) * 2.0) as i32;
        
        for i in 0..river_count {
            let river = self.generate_single_river(noise_gen, i);
            
            // Apply river to world chunks
            for &pos in &river.path {
                let chunk_pos = (
                    (pos.x as i32 / 16),
                    (pos.y as i32 / 16),
                    (pos.z as i32 / 16)
                );
                
                if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
                    let lx = pos.x.rem_euclid(16.0) as usize;
                    let ly = pos.y.rem_euclid(16.0) as usize;
                    let lz = pos.z.rem_euclid(16.0) as usize;
                    
                    // Create river bed and water
                    self.create_river_at_position(chunk, lx, ly, lz, &river);
                }
            }
            
            self.rivers.insert((i, 0, 0), river);
        }
    }

    fn generate_single_river(&self, noise_gen: &NoiseGenerator, seed: i32) -> RiverData {
        let mut path = Vec::new();
        let mut current_pos = Vec3::new(
            (seed as f32 * 1000.0).sin() * 200.0,
            60.0,
            (seed as f32 * 1000.0).cos() * 200.0
        );
        
        // Generate river path using noise
        for step in 0..500 {
            path.push(current_pos);
            
            // Use noise to determine river direction
            let angle = noise_gen.get_noise_octaves(
                current_pos.x as f64 * 0.01,
                current_pos.z as f64 * 0.01,
                step as f64 * 0.1,
                4
            ) * std::f64::consts::PI * 2.0;
            
            let next_direction = Vec3::new(
                angle.cos() as f32,
                0.0,
                angle.sin() as f32
            ).normalize();
            
            current_pos = current_pos + next_direction * 2.0;
            
            // Keep river at water level
            current_pos.y = 60.0;
            
            // Stop river if it goes too far from origin
            if current_pos.length() > 300.0 {
                break;
            }
        }
        
        RiverData {
            path,
            width: 4.0 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 2) * 2.0) as f32,
            depth: 3.0,
            flow_direction: Vec3::new(1.0, 0.0, 0.0),
            water_type: WaterType::River,
        }
    }

    fn create_river_at_position(&self, chunk: &mut Chunk, lx: usize, ly: usize, lz: usize, river: &RiverData) {
        let river_width = river.width as i32;
        let river_depth = river.depth as i32;
        
        // Create river bed (sand/gravel)
        for dx in -river_width..=river_width {
            for dz in -river_width..=river_width {
                let distance = (dx * dx + dz * dz) as f32;
                if distance <= (river_width * river_width) as f32 {
                    let tlx = lx as i32 + dx;
                    let tlz = lz as i32 + dz;
                    
                    if tlx >= 0 && tlx < 16 && tlz >= 0 && tlz < 16 {
                        let current = chunk.get_block(tlx as usize, ly as usize, tlz as usize);
                        if current == BlockType::Stone || current == BlockType::Dirt || current == BlockType::Grass {
                            // Replace with sand or gravel for river bed
                            let river_bed = if (tlx + tlz) % 3 == 0 {
                                BlockType::Gravel
                            } else {
                                BlockType::Sand
                            };
                            chunk.set_block(tlx as usize, ly as usize, tlz as usize, river_bed);
                        }
                    }
                }
            }
        }
        
        // Create water
        for dx in -river_width..=river_width {
            for dz in -river_width..=river_width {
                let distance = (dx * dx + dz * dz) as f32;
                if distance <= (river_width * river_width) as f32 {
                    let tlx = lx as i32 + dx;
                    let tlz = lz as i32 + dz;
                    
                    if tlx >= 0 && tlx < 16 && tlz >= 0 && tlz < 16 {
                        chunk.set_block(tlx as usize, ly as usize, tlz as usize, BlockType::Water);
                        chunk.is_empty = false;
                    }
                }
            }
        }
    }

    pub fn generate_oceans(&mut self, world: &mut World, noise_gen: &NoiseGenerator) {
        // Generate ocean islands using noise
        let island_count = 5 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 3) * 3.0) as i32;
        
        for i in 0..island_count {
            let island_pos = Vec3::new(
                (i as f32 * 200.0).sin() * 300.0,
                60.0,
                (i as f32 * 200.0).cos() * 300.0
            );
            
            self.oceans.islands.push(island_pos);
            
            // Generate island terrain
            self.generate_island(world, noise_gen, island_pos);
        }
    }

    fn generate_island(&self, world: &mut World, noise_gen: &NoiseGenerator, center: Vec3) {
        let island_radius = 80.0;
        let beach_width = self.oceans.beach_width;
        
        // Generate chunks around island center
        let cx_min = ((center.x - island_radius) / 16.0) as i32;
        let cx_max = ((center.x + island_radius) / 16.0) as i32;
        let cz_min = ((center.z - island_radius) / 16.0) as i32;
        let cz_max = ((center.z + island_radius) / 16.0) as i32;
        
        for cx in cx_min..=cx_max {
            for cz in cz_min..=cz_max {
                for cy in 0..(WORLD_HEIGHT / 16) {
                    let chunk_pos = (cx, cy, cz);
                    if !world.chunks.contains_key(&chunk_pos) {
                        self.generate_island_chunk(world, noise_gen, chunk_pos, center, island_radius, beach_width);
                    }
                }
            }
        }
    }

    fn generate_island_chunk(&self, world: &mut World, noise_gen: &NoiseGenerator, chunk_pos: (i32, i32, i32), center: Vec3, island_radius: f32, beach_width: f32) {
        let mut chunk = Chunk::new();
        let chunk_x_world = chunk_pos.0 * 16;
        let chunk_y_world = chunk_pos.1 * 16;
        let chunk_z_world = chunk_pos.2 * 16;
        
        let _distance_from_center = ((chunk_x_world as f32 - center.x).powi(2) + (chunk_z_world as f32 - center.z).powi(2)).sqrt();
        
        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = chunk_x_world + lx as i32;
                let wz = chunk_z_world + lz as i32;
                
                // Calculate distance from island center
                let distance = ((wx as f32 - center.x).powi(2) + (wz as f32 - center.z).powi(2)).sqrt();
                
                for ly in 0..CHUNK_SIZE_Y {
                    let y_world = chunk_y_world + ly as i32;
                    let mut block = BlockType::Water; // Default to water
                    
                    if distance < island_radius {
                        // Generate island terrain
                        let height = self.get_island_height(noise_gen, wx, y_world, wz, center, island_radius, beach_width);
                        
                        if height > self.water_level {
                            // Above water level - generate terrain
                            let density = noise_gen.get_density(wx, y_world, wz, 0.5, 0.5, 0.5);
                            
                            if density > 0.0 {
                                if distance < island_radius - beach_width {
                                    // Main island - grass/dirt/stone
                                    if y_world < 70 {
                                        block = BlockType::Grass;
                                    } else if y_world < 80 {
                                        block = BlockType::Dirt;
                                    } else {
                                        block = BlockType::Stone;
                                    }
                                } else {
                                    // Beach area - sand
                                    block = BlockType::Sand;
                                }
                            }
                        }
                    }
                    
                    // Set block
                    chunk.set_block(lx, ly, lz, block);
                    if block != BlockType::Air {
                        chunk.is_empty = false;
                    }
                }
            }
        }
        
        world.chunks.insert(chunk_pos, chunk);
    }

    fn get_island_height(&self, noise_gen: &NoiseGenerator, x: i32, y: i32, z: i32, center: Vec3, island_radius: f32, _beach_width: f32) -> f32 {
        let distance = ((x as f32 - center.x).powi(2) + (z as f32 - center.z).powi(2)).sqrt();
        
        if distance > island_radius {
            return self.water_level; // Ocean level
        }
        
        let normalized_distance = distance / island_radius;
        let height_factor = (1.0 - normalized_distance).max(0.0);
        
        // Generate height using noise
        let base_height = 70.0 + height_factor * 30.0;
        let noise = noise_gen.get_noise_octaves(x as f64 * 0.02, y as f64 * 0.02, z as f64 * 0.02, 4) as f32;
        
        base_height + noise * 10.0
    }

    pub fn update_water_flow(&mut self, world: &mut World) {
        // Update river flow directions based on terrain
        for (_, river) in self.rivers.iter_mut() {
            for i in 1..river.path.len() {
                let from = river.path[i - 1];
                let to = river.path[i];
                let flow_direction = (to - from).normalize();
                
                // Update flow direction for this segment
                river.flow_direction = flow_direction;
                
                // Apply flow to water blocks
                let chunk_pos = (
                    (to.x as i32 / 16),
                    (to.y as i32 / 16),
                    (to.z as i32 / 16)
                );
                
                if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
                    let lx = to.x.rem_euclid(16.0) as usize;
                    let ly = to.y.rem_euclid(16.0) as usize;
                    let lz = to.z.rem_euclid(16.0) as usize;
                    
                    if chunk.get_block(lx, ly, lz) == BlockType::Water {
                        // Update water flow direction (would be used for rendering)
                        // This could be stored in a separate water flow map
                    }
                }
            }
        }
    }

    pub fn is_water_at(&self, pos: BlockPos) -> bool {
        // Check if position is in water (river or ocean)
        let chunk_pos = (
            pos.x / 16,
            pos.y / 16,
            pos.z / 16
        );
        
        // Check if in river
        for (_, river) in &self.rivers {
            for river_pos in &river.path {
                let distance = (*river_pos - Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)).length();
                if distance < river.width {
                    return true;
                }
            }
        }
        
        // Check if in ocean (below water level and not in land)
        if pos.y < self.water_level as i32 {
            // Would need to check if this position is land or water
            // For now, assume below water level is ocean
            return true;
        }
        
        false
    }

    pub fn get_water_level(&self) -> f32 {
        self.water_level
    }

    pub fn set_water_level(&mut self, level: f32) {
        self.water_level = level;
        self.oceans.ocean_level = level;
    }
}
