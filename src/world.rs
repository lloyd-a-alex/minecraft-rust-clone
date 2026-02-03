use std::collections::HashMap;
use crate::noise_gen::NoiseGenerator;
use crate::player::Player;
use glam::Vec3;
use serde::{Serialize, Deserialize};

struct SimpleRng { state: u64 }
impl SimpleRng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) ^ self.state) as u32 as f32 / u32::MAX as f32
    }
    fn gen_range(&mut self, min: f32, max: f32) -> f32 { min + (max - min) * self.next_f32() }
}

pub const CHUNK_SIZE_X: usize = 16;
pub const CHUNK_SIZE_Z: usize = 16;
pub const CHUNK_SIZE_Y: usize = 128;
pub const WATER_LEVEL: i32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    Air = 0, Grass = 1, Dirt = 2, Stone = 3, Wood = 4, Leaves = 5,
    Snow = 6, Sand = 7, Bedrock = 8, Water = 9,
    CoalOre = 10, IronOre = 11, GoldOre = 12, DiamondOre = 13,
    Planks = 14, Stick = 15, Cobblestone = 16,
    IronIngot = 17, GoldIngot = 18, Diamond = 19, Torch = 20,
    WoodPickaxe = 21, StonePickaxe = 22, IronPickaxe = 23, GoldPickaxe = 24, DiamondPickaxe = 25,
}

#[allow(dead_code)]
impl BlockType {
    pub fn is_solid(&self) -> bool { !matches!(self, BlockType::Air | BlockType::Water | BlockType::Torch) }
    pub fn is_water(&self) -> bool { *self == BlockType::Water }
    pub fn get_texture_indices(&self) -> (u32, u32, u32) {
        match self {
            BlockType::Grass => (1, 2, 1),
            BlockType::Dirt => (2, 2, 2),
            BlockType::Stone | BlockType::Cobblestone => (3, 3, 3),
            BlockType::Wood => (4, 4, 4),
            BlockType::Leaves => (5, 5, 5),
            BlockType::Snow => (6, 2, 6),
            BlockType::Sand => (7, 7, 7),
            BlockType::Bedrock => (8, 8, 8),
            BlockType::Water => (9, 9, 9),
            BlockType::CoalOre => (10, 10, 10),
            BlockType::IronOre | BlockType::IronIngot => (11, 11, 11),
            BlockType::GoldOre | BlockType::GoldIngot => (12, 12, 12),
            BlockType::DiamondOre | BlockType::Diamond => (13, 13, 13),
            BlockType::Planks => (14, 14, 14),
            BlockType::Torch => (10, 10, 10),
            _ => (1, 1, 1),
        }
    }
}

pub struct Chunk { pub blocks: Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> }
impl Chunk {
    pub fn new() -> Self { Chunk { blocks: Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]) } }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z { return BlockType::Air; }
        self.blocks[x][y][z]
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z { self.blocks[x][y][z] = block; }
    }
}

#[allow(dead_code)]
pub struct ItemEntity {
    pub position: Vec3, pub velocity: Vec3, pub item_type: BlockType, pub count: u8,
    pub pickup_delay: f32, pub lifetime: f32, pub rotation: f32, pub bob_offset: f32,
}
pub struct RemotePlayer { pub id: u32, pub position: Vec3, pub rotation: f32 }

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub entities: Vec<ItemEntity>,
    pub remote_players: Vec<RemotePlayer>,
    seed: u32,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let mut world = World { chunks: HashMap::new(), entities: Vec::new(), remote_players: Vec::new(), seed };
        world.generate_terrain();
        world
    }

    fn generate_terrain(&mut self) {
        let noise_gen = NoiseGenerator::new(self.seed);
        let render_distance = 6; 

        for cx in -render_distance..=render_distance {
            for cz in -render_distance..=render_distance {
                let mut chunk = Chunk::new();
                let chunk_x_world = cx * (CHUNK_SIZE_X as i32);
                let chunk_z_world = cz * (CHUNK_SIZE_Z as i32);
                
                for lx in 0..CHUNK_SIZE_X {
                    for lz in 0..CHUNK_SIZE_Z {
                        let wx = chunk_x_world + lx as i32;
                        let wz = chunk_z_world + lz as i32;
                        let mut height = noise_gen.get_height(wx, wz);
                        
                        let river_val = noise_gen.get_river_noise(wx, wz);
                        let river_width = 0.05;
                        let mut is_river = false;

                        if river_val.abs() < river_width {
                            let depth_factor = (river_width - river_val.abs()) / river_width;
                            let river_bed = (WATER_LEVEL - 2) as i32;
                            let carved_height = (height as f32 * (1.0 - depth_factor as f32) + river_bed as f32 * depth_factor as f32) as i32;
                            if carved_height < height { height = carved_height; is_river = true; }
                        }

                        let biome = noise_gen.get_biome(wx, wz, height);
                        for y in 0..CHUNK_SIZE_Y {
                            let y_i32 = y as i32;
                            let mut block = BlockType::Air;
                            if y_i32 <= height {
                                if y_i32 == 0 { block = BlockType::Bedrock; }
                                else if y_i32 <= height - 4 { block = BlockType::Stone; }
                                else {
                                    if is_river { block = BlockType::Sand; } 
                                    else {
                                        match biome { "desert" => block = BlockType::Sand, "snow" => block = BlockType::Snow, _ => {
                                            if y_i32 < WATER_LEVEL + 2 { block = BlockType::Sand; } else { block = BlockType::Grass; }
                                        }} 
                                    }
                                }
                            } else if y_i32 <= WATER_LEVEL { block = BlockType::Water; }
                            
                            if block != BlockType::Air { chunk.set_block(lx, y, lz, block); }
                        }
                    }
                }
                self.chunks.insert((cx, cz), chunk);
            }
        }
    }

    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        let cx = (pos.x as f32 / CHUNK_SIZE_X as f32).floor() as i32;
        let cz = (pos.z as f32 / CHUNK_SIZE_Z as f32).floor() as i32;
        let lx = (pos.x - cx * CHUNK_SIZE_X as i32) as usize;
        let lz = (pos.z - cz * CHUNK_SIZE_Z as i32) as usize;
        if let Some(chunk) = self.chunks.get(&(cx, cz)) { chunk.get_block(lx, pos.y as usize, lz) } else { BlockType::Air }
    }

    pub fn place_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<(i32, i32)> {
        let cx = (pos.x as f32 / CHUNK_SIZE_X as f32).floor() as i32;
        let cz = (pos.z as f32 / CHUNK_SIZE_Z as f32).floor() as i32;
        let lx = (pos.x - cx * CHUNK_SIZE_X as i32) as usize;
        let lz = (pos.z - cz * CHUNK_SIZE_Z as i32) as usize;
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            chunk.set_block(lx, pos.y as usize, lz, block);
            return vec![(cx, cz)];
        }
        vec![]
    }

    pub fn break_block(&mut self, pos: BlockPos) -> Vec<(i32, i32)> {
        let cx = (pos.x as f32 / CHUNK_SIZE_X as f32).floor() as i32;
        let cz = (pos.z as f32 / CHUNK_SIZE_Z as f32).floor() as i32;
        let lx = (pos.x - cx * CHUNK_SIZE_X as i32) as usize;
        let lz = (pos.z - cz * CHUNK_SIZE_Z as i32) as usize;
        
        let mut affected = vec![];
        let mut drop_item = BlockType::Air;

        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            drop_item = chunk.get_block(lx, pos.y as usize, lz);
            if drop_item != BlockType::Air {
                chunk.set_block(lx, pos.y as usize, lz, BlockType::Air);
                affected.push((cx, cz));
            } else { return vec![]; }
        }

        if drop_item != BlockType::Air {
             let mut rng = SimpleRng::new(pos.x as u64 ^ pos.y as u64 ^ pos.z as u64);
             let position = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
             let velocity = Vec3::new(rng.gen_range(-2.0, 2.0), rng.gen_range(2.0, 4.0), rng.gen_range(-2.0, 2.0));
             self.entities.push(ItemEntity { position, velocity, item_type: drop_item, count: 1, pickup_delay: 1.0, lifetime: 300.0, rotation: 0.0, bob_offset: rng.next_f32() * 10.0 });
        }
        affected
    }

    pub fn raycast(&self, start: Vec3, dir: Vec3, max_dist: f32) -> Option<(BlockPos, BlockPos)> {
        let mut t = 0.0;
        let step = 0.02; // Small step for accuracy
        let mut last_pos = BlockPos { x: start.x.floor() as i32, y: start.y.floor() as i32, z: start.z.floor() as i32 };

        while t < max_dist {
            let p = start + dir * t;
            let bp = BlockPos { x: p.x.floor() as i32, y: p.y.floor() as i32, z: p.z.floor() as i32 };
            // Ensure we don't pick the block our head is currently inside unless we moved
            if bp != last_pos {
                let block = self.get_block(bp);
                if block.is_solid() { return Some((bp, last_pos)); }
                last_pos = bp;
            }
            t += step;
        }
        None
    }

    pub fn update_entities(&mut self, dt: f32, player: &mut Player) {
        let entities = std::mem::take(&mut self.entities);
        let mut retained = Vec::new();
        
        for mut entity in entities {
            entity.lifetime -= dt;
            if entity.lifetime <= 0.0 { continue; }
            if entity.pickup_delay > 0.0 { entity.pickup_delay -= dt; }
            
            entity.velocity.y -= 25.0 * dt;
            let next_pos = entity.position + entity.velocity * dt;
            
            // Simple ground collision
            let block_pos_below = BlockPos { x: next_pos.x.floor() as i32, y: next_pos.y.floor() as i32, z: next_pos.z.floor() as i32 };
            let block_below = self.get_block(block_pos_below);
            
            if block_below.is_solid() {
                // Land on top
                if entity.velocity.y < 0.0 {
                    entity.velocity.y = 0.0;
                    entity.velocity.x *= 0.8; 
                    entity.velocity.z *= 0.8;
                    // Snap exactly to top of block + epsilon
                    entity.position.y = (block_pos_below.y as f32 + 1.001); 
                    entity.position.x = next_pos.x;
                    entity.position.z = next_pos.z;
                }
            } else {
                entity.position = next_pos;
            }
            
            // Pickup logic
            let dist_sq = entity.position.distance_squared(player.position);
            if dist_sq < 3.0 * 3.0 && entity.pickup_delay <= 0.0 {
                let dir = (player.position - entity.position).normalize();
                entity.position += dir * 8.0 * dt; 
                if dist_sq < 1.0 * 1.0 {
                    if player.inventory.add_item(entity.item_type) { continue; }
                }
            }
            retained.push(entity);
        }
        self.entities = retained;
    }
}