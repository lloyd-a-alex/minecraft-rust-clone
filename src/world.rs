use std::collections::{HashMap, VecDeque, HashSet};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    Air = 0, Grass = 1, Dirt = 2, Stone = 3, Wood = 4, Leaves = 5, Snow = 6, Sand = 7, Bedrock = 8, Water = 9,
    CoalOre = 10, IronOre = 11, GoldOre = 12, DiamondOre = 13,
    Planks = 14, Stick = 15, Cobblestone = 16, IronIngot = 17, GoldIngot = 18, Diamond = 19, Torch = 20,
    WoodPickaxe = 21, StonePickaxe = 22, IronPickaxe = 23, GoldPickaxe = 24, DiamondPickaxe = 25,
    WoodAxe = 26, StoneAxe = 27, IronAxe = 28, GoldAxe = 29, DiamondAxe = 30,
    WoodShovel = 31, StoneShovel = 32, IronShovel = 33, GoldShovel = 34, DiamondShovel = 35,
    WoodSword = 36, StoneSword = 37, IronSword = 38, GoldSword = 39, DiamondSword = 40,
    CraftingTable = 100, Furnace = 101,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water | BlockType::Torch | BlockType::Stick | BlockType::IronIngot | BlockType::GoldIngot | BlockType::Diamond) && !self.is_tool()
    }
    pub fn is_water(&self) -> bool { matches!(self, BlockType::Water) }
    pub fn is_tool(&self) -> bool { (*self as u8) >= 21 && (*self as u8) <= 40 }
    pub fn is_item(&self) -> bool { matches!(self, BlockType::Stick | BlockType::IronIngot | BlockType::GoldIngot | BlockType::Diamond) }
    
    pub fn get_texture_indices(&self) -> (u32, u32, u32) {
        match self {
            BlockType::Grass => (0, 2, 1), BlockType::Dirt => (2, 2, 2), BlockType::Stone => (3, 3, 3),
            BlockType::Wood => (4, 4, 4), BlockType::Leaves => (5, 5, 5), BlockType::Snow => (6, 6, 6),
            BlockType::Sand => (7, 7, 7), BlockType::Bedrock => (8, 8, 8), BlockType::Water => (9, 9, 9),
            BlockType::CoalOre => (10, 10, 10), BlockType::IronOre => (11, 11, 11), BlockType::GoldOre => (12, 12, 12), BlockType::DiamondOre => (13, 13, 13),
            BlockType::Planks => (14, 14, 14), BlockType::Stick => (15, 15, 15), BlockType::Cobblestone => (16, 16, 16),
            BlockType::Torch => (20, 20, 20), BlockType::CraftingTable => (21, 21, 14), // Top=21, Side=21, Bottom=14
            t if t.is_tool() => { let i = *t as u32; (i, i, i) }
            _ => (0, 0, 0),
        }
    }

    pub fn get_display_name(&self) -> &str {
        match self {
            BlockType::Air => "Air", BlockType::Grass => "Grass", BlockType::Dirt => "Dirt",
            BlockType::Stone => "Stone", BlockType::Wood => "Log", BlockType::Leaves => "Leaves",
            BlockType::Snow => "Snow", BlockType::Sand => "Sand", BlockType::Bedrock => "Bedrock",
            BlockType::Water => "Water", BlockType::CoalOre => "Coal Ore", BlockType::IronOre => "Iron Ore",
            BlockType::GoldOre => "Gold Ore", BlockType::DiamondOre => "Diamond Ore",
            BlockType::Planks => "Wooden Plank", BlockType::Stick => "Stick", BlockType::Cobblestone => "Cobblestone",
            BlockType::IronIngot => "Iron Ingot", BlockType::GoldIngot => "Gold Ingot", BlockType::Diamond => "Diamond",
            BlockType::Torch => "Torch", BlockType::CraftingTable => "Crafting Table", BlockType::Furnace => "Furnace",
            t if t.is_tool() => "Tool", // Simplified, can expand
            _ => "Unknown Block"
        }
    }

    pub fn get_hardness(&self) -> f32 {
        match self {
            BlockType::Bedrock | BlockType::Water | BlockType::Air => -1.0,
            BlockType::Leaves => 0.2, BlockType::Sand | BlockType::Dirt | BlockType::Grass => 0.5,
            BlockType::Wood | BlockType::Planks | BlockType::CraftingTable => 2.0,
            BlockType::Stone | BlockType::Cobblestone | BlockType::CoalOre => 3.0,
            BlockType::IronOre | BlockType::GoldOre | BlockType::DiamondOre => 4.5,
            _ => 1.0,
        }
    }
    
    pub fn get_best_tool_type(&self) -> &'static str {
        match self {
            BlockType::Stone | BlockType::Cobblestone | BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre | BlockType::DiamondOre | BlockType::Furnace => "pickaxe",
            BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Snow => "shovel",
            BlockType::Wood | BlockType::Planks | BlockType::CraftingTable | BlockType::Leaves => "axe",
            _ => "none",
        }
    }
    
    pub fn get_tool_class(&self) -> &'static str {
        let u = *self as u8;
        if u >= 21 && u <= 25 { "pickaxe" } else if u >= 26 && u <= 30 { "axe" } else if u >= 31 && u <= 35 { "shovel" } else if u >= 36 && u <= 40 { "sword" } else { "none" }
    }
    
    pub fn get_tool_speed(&self) -> f32 {
        let u = *self as u8;
        if u % 5 == 1 { 2.0 } else if u % 5 == 2 { 4.0 } else if u % 5 == 3 { 6.0 } else if u % 5 == 4 { 8.0 } else if u % 5 == 0 { 10.0 } else { 1.0 }
    }
}

pub struct Chunk { pub blocks: Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> }
impl Chunk {
    pub fn new() -> Self { Chunk { blocks: Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]) } }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType { if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z { return BlockType::Air; } self.blocks[x][y][z] }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) { if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z { self.blocks[x][y][z] = block; } }
}
#[allow(dead_code)]
pub struct ItemEntity { pub position: Vec3, pub velocity: Vec3, pub item_type: BlockType, pub count: u8, pub pickup_delay: f32, pub lifetime: f32, pub rotation: f32, pub bob_offset: f32 }
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
                        let wx = chunk_x_world + lx as i32; let wz = chunk_z_world + lz as i32;
                        let mut height = noise_gen.get_height(wx, wz);
                        let river_val = noise_gen.get_river_noise(wx, wz);
                        let mut is_river = false;
                        if river_val.abs() < 0.08 {
                            let depth_factor = (0.08 - river_val.abs()) / 0.08;
                            height = (height as f32 * (1.0 - depth_factor as f32) + (WATER_LEVEL - 2) as f32 * depth_factor as f32) as i32;
                            is_river = true;
                        }
                        let biome = noise_gen.get_biome(wx, wz, height);
                        for y in 0..CHUNK_SIZE_Y {
                            let y_i32 = y as i32;
                            let mut block = BlockType::Air;
                            if y_i32 <= height {
                                if y_i32 == 0 { block = BlockType::Bedrock; }
                                else if y_i32 < height - 3 {
                                    let mut rng = SimpleRng::new((wx as u64).wrapping_mul(73856093) ^ (wz as u64).wrapping_mul(19349663) ^ (y_i32 as u64));
                                    let r = rng.next_f32();
                                    if r < 0.01 { block = BlockType::CoalOre; } else if r < 0.015 && y_i32 < 40 { block = BlockType::IronOre; } else if r < 0.002 && y_i32 < 20 { block = BlockType::GoldOre; } else if r < 0.001 && y_i32 < 12 { block = BlockType::DiamondOre; } else { block = BlockType::Stone; }
                                } else if y_i32 < height { block = if is_river || biome == "desert" { BlockType::Sand } else { BlockType::Dirt }; }
                                else { block = if is_river { BlockType::Sand } else if height >= 85 { BlockType::Snow } else if biome == "desert" { BlockType::Sand } else if height <= WATER_LEVEL + 2 { BlockType::Sand } else { BlockType::Grass }; }
                            } else if y_i32 <= WATER_LEVEL { block = BlockType::Water; }
                            if block != BlockType::Air { chunk.set_block(lx, y, lz, block); }
                        }
                    }
                }
                self.chunks.insert((cx, cz), chunk);
            }
        }
        // Trees
        for cx in -render_distance..=render_distance {
            for cz in -render_distance..=render_distance {
                let chunk_x_world = cx * (CHUNK_SIZE_X as i32); let chunk_z_world = cz * (CHUNK_SIZE_Z as i32);
                for lx in 0..CHUNK_SIZE_X {
                    for lz in 0..CHUNK_SIZE_Z {
                        let wx = chunk_x_world + lx as i32; let wz = chunk_z_world + lz as i32;
                        let height = noise_gen.get_height(wx, wz);
                        if noise_gen.get_river_noise(wx, wz).abs() < 0.15 { continue; }
                        let biome = noise_gen.get_biome(wx, wz, height);
                        let mut rng = SimpleRng::new((wx as u64).wrapping_mul(self.seed as u64) ^ (wz as u64));
                        let spawn_chance = if biome == "forest" { 0.02 } else if biome == "plains" { 0.002 } else { 0.0 };
                        if rng.next_f32() < spawn_chance && height > WATER_LEVEL && height < 50 && (self.get_block(BlockPos{x:wx, y:height, z:wz}) == BlockType::Grass) {
                            for i in 1..=5 { self.set_block_world(BlockPos{x:wx, y:height+i, z:wz}, BlockType::Wood); }
                            for ly in (height+3)..(height+6) {
                                for dx in -2..=2 { for dz in -2..=2 {
                                    if (dx*dx + dz*dz) > 5 || rng.next_f32() > 0.9 { continue; }
                                    let lp = BlockPos{x:wx+dx, y:ly, z:wz+dz};
                                    if self.get_block(lp) == BlockType::Air { self.set_block_world(lp, BlockType::Leaves); }
                                }}
                            }
                            self.set_block_world(BlockPos{x:wx, y:height+6, z:wz}, BlockType::Leaves);
                        }
                    }
                }
            }
        }
    }

    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        let cx = pos.x.div_euclid(CHUNK_SIZE_X as i32); let cz = pos.z.div_euclid(CHUNK_SIZE_Z as i32);
        let lx = pos.x.rem_euclid(CHUNK_SIZE_X as i32) as usize; let lz = pos.z.rem_euclid(CHUNK_SIZE_Z as i32) as usize;
        if let Some(chunk) = self.chunks.get(&(cx, cz)) { chunk.get_block(lx, pos.y as usize, lz) } else { BlockType::Air }
    }
    pub fn set_block_world(&mut self, pos: BlockPos, block: BlockType) {
        let cx = pos.x.div_euclid(CHUNK_SIZE_X as i32); let cz = pos.z.div_euclid(CHUNK_SIZE_Z as i32);
        let lx = pos.x.rem_euclid(CHUNK_SIZE_X as i32) as usize; let lz = pos.z.rem_euclid(CHUNK_SIZE_Z as i32) as usize;
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) { chunk.set_block(lx, pos.y as usize, lz, block); }
    }
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_dist: f32) -> Option<(BlockPos, BlockPos)> {
        let mut x = origin.x.floor() as i32; let mut y = origin.y.floor() as i32; let mut z = origin.z.floor() as i32;
        let step_x = if direction.x > 0.0 { 1 } else { -1 }; let step_y = if direction.y > 0.0 { 1 } else { -1 }; let step_z = if direction.z > 0.0 { 1 } else { -1 };
        let mut t_max_x = if direction.x > 0.0 { (x as f32 + 1.0 - origin.x) / direction.x } else { (x as f32 - origin.x) / direction.x };
        let mut t_max_y = if direction.y > 0.0 { (y as f32 + 1.0 - origin.y) / direction.y } else { (y as f32 - origin.y) / direction.y };
        let mut t_max_z = if direction.z > 0.0 { (z as f32 + 1.0 - origin.z) / direction.z } else { (z as f32 - origin.z) / direction.z };
        let t_delta_x = (1.0 / direction.x).abs(); let t_delta_y = (1.0 / direction.y).abs(); let t_delta_z = (1.0 / direction.z).abs();
        let mut t = 0.0; let mut last_pos = BlockPos { x, y, z };
        while t < max_dist {
            let current_pos = BlockPos { x, y, z };
            if self.get_block(current_pos).is_solid() { return Some((current_pos, last_pos)); }
            last_pos = current_pos;
            if t_max_x < t_max_y { if t_max_x < t_max_z { x += step_x; t = t_max_x; t_max_x += t_delta_x; } else { z += step_z; t = t_max_z; t_max_z += t_delta_z; } }
            else { if t_max_y < t_max_z { y += step_y; t = t_max_y; t_max_y += t_delta_y; } else { z += step_z; t = t_max_z; t_max_z += t_delta_z; } }
        }
        None
    }
    pub fn get_affected_chunks(&self, pos: BlockPos) -> Vec<(i32, i32)> {
        let cx = pos.x.div_euclid(CHUNK_SIZE_X as i32);
        let cz = pos.z.div_euclid(CHUNK_SIZE_Z as i32);
        let lx = pos.x.rem_euclid(CHUNK_SIZE_X as i32);
        let lz = pos.z.rem_euclid(CHUNK_SIZE_Z as i32);
        let mut u = vec![(cx, cz)];
        if lx == 0 { u.push((cx - 1, cz)); } else if lx == 15 { u.push((cx + 1, cz)); }
        if lz == 0 { u.push((cx, cz - 1)); } else if lz == 15 { u.push((cx, cz + 1)); }
        u
    }

    pub fn break_block(&mut self, pos: BlockPos) -> Vec<(i32, i32)> {
        let block_type = self.get_block(pos);
        if block_type != BlockType::Air && block_type != BlockType::Bedrock && !block_type.is_water() {
             let mut rng = SimpleRng::new(pos.x as u64 ^ pos.z as u64 ^ pos.y as u64);
             let velocity = Vec3::new(rng.gen_range(-2.0, 2.0), 4.0, rng.gen_range(-2.0, 2.0));
             let drop_item = match block_type { BlockType::Stone => BlockType::Cobblestone, BlockType::CoalOre => BlockType::Coal, BlockType::IronOre => BlockType::IronOre, BlockType::DiamondOre => BlockType::Diamond, BlockType::Grass => BlockType::Dirt, _ => block_type };
             self.entities.push(ItemEntity { position: Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5), velocity, item_type: drop_item, count: 1, pickup_delay: 1.0, lifetime: 300.0, rotation: 0.0, bob_offset: rng.next_f32() * 10.0 });
        }
self.set_block_world(pos, BlockType::Air);
        let mut c = self.trigger_water_update(pos);
        c.extend(self.get_affected_chunks(pos));
        c.sort(); c.dedup(); c
    }
    pub fn place_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<(i32, i32)> { 
        self.set_block_world(pos, block); 
        let mut c = self.trigger_water_update(pos);
        c.extend(self.get_affected_chunks(pos));
        c.sort(); c.dedup(); c
    }
    fn trigger_water_update(&mut self, start_pos: BlockPos) -> Vec<(i32, i32)> {
        let mut updates = Vec::new();
        let cx = start_pos.x.div_euclid(CHUNK_SIZE_X as i32); let cz = start_pos.z.div_euclid(CHUNK_SIZE_Z as i32);
        // Only update affected chunks
        updates.push((cx, cz));
        
        // For water updates, limit to immediate neighbors
        let mut queue = VecDeque::new(); 
        queue.push_back(start_pos);
        let b = self.get_block(start_pos);
        if !b.is_water() { 
            for (dx, dy, dz) in &[(0,1,0), (0,-1,0), (1,0,0), (-1,0,0), (0,0,1), (0,0,-1)] { 
                let check_pos = BlockPos{x:start_pos.x+dx, y:start_pos.y+dy, z:start_pos.z+dz};
                if self.get_block(check_pos).is_water() { 
                    queue.push_back(check_pos); 
                } 
            } 
        }
        let mut visited = HashSet::new(); 
        let mut steps = 0;
        let max_steps = 50; // Reduced from 200 to prevent lag
        while let Some(pos) = queue.pop_front() {
            if steps > max_steps { 
                log::debug!("Water update reached max steps ({})", max_steps);
                break; 
            }
            if !visited.insert(pos) { continue; }
            steps += 1;
            let current = self.get_block(pos);
            if current.is_water() {
                let below = BlockPos { x: pos.x, y: pos.y - 1, z: pos.z };
                if self.get_block(below) == BlockType::Air {
                    self.set_block_world(below, BlockType::Water); updates.push((below.x.div_euclid(CHUNK_SIZE_X as i32), below.z.div_euclid(CHUNK_SIZE_Z as i32))); queue.push_back(below);
                } else if self.get_block(below).is_solid() || self.get_block(below).is_water() {
                    for (dx, dz) in &[(1,0), (-1,0), (0,1), (0,-1)] {
                        let side = BlockPos { x: pos.x + dx, y: pos.y, z: pos.z + dz };
                        if self.get_block(side) == BlockType::Air {
                            let lvl = current.get_water_level();
                            if lvl > 1 {
                                let new_blk = match lvl - 1 { 7 => BlockType::Water7, 6 => BlockType::Water6, _ => BlockType::Water1 };
                                self.set_block_world(side, new_blk); updates.push((side.x.div_euclid(CHUNK_SIZE_X as i32), side.z.div_euclid(CHUNK_SIZE_Z as i32))); queue.push_back(side);
                            }
                        }
                    }
                }
            }
        }
        updates.sort(); updates.dedup(); updates
    }
    pub fn update_entities(&mut self, dt: f32, player: &mut Player) {
        let entities = std::mem::take(&mut self.entities);
        let mut retained = Vec::new();
        for mut entity in entities {
            entity.lifetime -= dt; if entity.lifetime <= 0.0 { continue; }
            if entity.pickup_delay > 0.0 { entity.pickup_delay -= dt; }
            entity.velocity.y -= 25.0 * dt;
            let next_pos = entity.position + entity.velocity * dt;
            let block_below = self.get_block(BlockPos { x: next_pos.x.floor() as i32, y: next_pos.y.floor() as i32, z: next_pos.z.floor() as i32 });
            if block_below.is_solid() {
                entity.velocity.y = 0.0; entity.velocity.x *= 0.85; entity.velocity.z *= 0.85;
                entity.position.y = (next_pos.y.floor() as f32 + 1.0).max(next_pos.y); entity.position.x = next_pos.x; entity.position.z = next_pos.z;
            } else { entity.position = next_pos; }
            let dist_sq = entity.position.distance_squared(player.position);
            if dist_sq < 9.0 && entity.pickup_delay <= 0.0 {
                let dir = (player.position - entity.position).normalize(); entity.position += dir * 10.0 * dt;
                if dist_sq < 2.25 { 
                    if player.inventory.add_item(entity.item_type) { 
                        log::info!("🎁 Picked up {:?}", entity.item_type);
                        continue; 
                    } 
                }
            }
            retained.push(entity);
        }
        self.entities = retained;
    }
}