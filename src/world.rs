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
    CoalOre = 10, IronOre = 11, GoldOre = 12, DiamondOre = 13, RedstoneOre = 125, LapisOre = 126,
    Planks = 14, Stick = 15, Cobblestone = 16, IronIngot = 17, GoldIngot = 18, Diamond = 19, Torch = 20,
    WoodPickaxe = 21, StonePickaxe = 22, IronPickaxe = 23, GoldPickaxe = 24, DiamondPickaxe = 25,
    WoodAxe = 26, StoneAxe = 27, IronAxe = 28, GoldAxe = 29, DiamondAxe = 30,
    WoodShovel = 31, StoneShovel = 32, IronShovel = 33, GoldShovel = 34, DiamondShovel = 35,
    WoodSword = 36, StoneSword = 37, IronSword = 38, GoldSword = 39, DiamondSword = 40,
    WoodHoe = 41, StoneHoe = 42, IronHoe = 43, GoldHoe = 44, DiamondHoe = 45,
    BucketEmpty = 46, BucketWater = 47,
    FarmlandDry = 48, FarmlandWet = 49,
    Gravel = 50, Clay = 51, Sandstone = 52, Obsidian = 53, Cactus = 54,
    Coal = 76, IronIngotItem = 77, GoldIngotItem = 78, DiamondItem = 79, // Added missing items
    GoldBlock = 120, IronBlock = 121, DiamondBlock = 122,
    Ice = 501, Mycelium = 502, LilyPad = 503, Vine = 504,
    Rose = 55, Dandelion = 56, DeadBush = 57, TallGrass = 58, Sugarcane = 59,
OakSapling = 60, Glass = 61, Bookshelf = 62, TNT = 63, Pumpkin = 64, Melon = 65,
    BrickBlock = 66, MossyCobble = 67, Lava = 70, Fire = 71,
    SpruceWood = 72, SpruceLeaves = 73, BirchWood = 74, BirchLeaves = 75,
    Wheat0 = 85, Wheat1 = 86, Wheat2 = 87, Wheat3 = 88, Wheat4 = 89, Wheat5 = 90, Wheat6 = 91, Wheat7 = 92,
    Cloud = 95,
    CraftingTable = 100, Furnace = 101, FurnaceActive = 102, Chest = 103,
    ChestLeft = 104, ChestRight = 105,
    WheatSeeds = 110, Wheat = 111, Bread = 112, Apple = 113, Porkchop = 114, CookedPorkchop = 115,
}

impl BlockType {
    pub fn get_water_level(&self) -> u8 { if *self == BlockType::Water { 8 } else { 0 } }
pub fn is_transparent(&self) -> bool { 
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Leaves | BlockType::SpruceLeaves | BlockType::BirchLeaves | 
                       BlockType::Torch | BlockType::Fire | BlockType::Glass | BlockType::Rose | BlockType::Dandelion | 
                       BlockType::DeadBush | BlockType::TallGrass | BlockType::OakSapling | BlockType::Sugarcane | 
                       BlockType::Cactus | BlockType::Ice | BlockType::LilyPad | BlockType::Vine | BlockType::Wheat) 
    }


    pub fn is_cross_model(&self) -> bool {
        matches!(self, BlockType::Rose | BlockType::Dandelion | BlockType::DeadBush | BlockType::TallGrass | BlockType::OakSapling | BlockType::Sugarcane)
    }
pub fn is_water(&self) -> bool { matches!(self, BlockType::Water) }

pub fn is_solid(&self) -> bool {
    !matches!(self, BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Rose | BlockType::Dandelion | BlockType::DeadBush | BlockType::TallGrass | BlockType::OakSapling | BlockType::Sugarcane | BlockType::Fire)
}

pub fn is_tool(&self) -> bool { (*self as u8) >= 21 && (*self as u8) <= 40 }
    pub fn is_item(&self) -> bool { matches!(self, BlockType::Coal | BlockType::Stick | BlockType::IronIngot | BlockType::GoldIngot | BlockType::Diamond | BlockType::Wheat | BlockType::Bread | BlockType::Apple | BlockType::Porkchop | BlockType::CookedPorkchop) }
    
pub fn get_texture_indices(&self) -> (u32, u32, u32) {
match self {
            BlockType::Grass => (0, 2, 1), BlockType::Dirt => (2, 2, 2), BlockType::Stone => (3, 3, 3),
            BlockType::Wood => (4, 4, 4), BlockType::Leaves => (5, 5, 5), BlockType::Snow => (6, 6, 6),
            BlockType::Sand => (7, 7, 7), BlockType::Bedrock => (8, 8, 8), BlockType::Water => (9, 9, 9),
            BlockType::Lava => (200, 200, 200), BlockType::Fire => (201, 201, 201),
            BlockType::SpruceWood => (202, 202, 202), BlockType::SpruceLeaves => (203, 203, 203),
            BlockType::BirchWood => (204, 204, 204), BlockType::BirchLeaves => (205, 205, 205),
            BlockType::CoalOre => (10, 10, 10), BlockType::IronOre => (11, 11, 11), BlockType::GoldOre => (12, 12, 12), BlockType::DiamondOre => (13, 13, 13),
            BlockType::RedstoneOre => (22, 22, 22), BlockType::LapisOre => (23, 23, 23),
            BlockType::Planks => (14, 14, 14), BlockType::Stick => (15, 15, 15), BlockType::Cobblestone => (16, 16, 16),
            BlockType::Torch => (20, 20, 20), BlockType::CraftingTable => (21, 25, 14), 
            BlockType::Furnace => (26, 27, 26), BlockType::Chest => (28, 29, 28),
            BlockType::Gravel => (30, 30, 30), BlockType::Clay => (31, 31, 31), BlockType::Sandstone => (32, 33, 32),
            BlockType::Obsidian => (34, 34, 34), BlockType::Cactus => (35, 36, 35),
            BlockType::Ice => (60, 60, 60), BlockType::LilyPad => (61, 61, 61), 
            BlockType::Mycelium => (62, 2, 63), // Top, Bot, Side
            BlockType::Vine => (64, 64, 64),
            BlockType::Rose => (37, 37, 37), BlockType::Dandelion => (38, 38, 38), BlockType::DeadBush => (39, 39, 39),
            BlockType::TallGrass => (45, 45, 45), BlockType::Sugarcane => (46, 46, 46), BlockType::OakSapling => (47, 47, 47),
            BlockType::Glass => (48, 48, 48), BlockType::Bookshelf => (14, 49, 14), 
BlockType::TNT => (50, 51, 50), BlockType::Pumpkin => (52, 53, 52), BlockType::Melon => (54, 55, 54),
            BlockType::GoldBlock => (120, 120, 120), BlockType::IronBlock => (121, 121, 121), BlockType::DiamondBlock => (122, 122, 122),
            BlockType::FarmlandDry => (123, 123, 123), BlockType::FarmlandWet => (124, 124, 124),
            BlockType::BrickBlock => (56, 56, 56), BlockType::MossyCobble => (57, 57, 57),
            BlockType::Wheat0 => (220, 220, 220), BlockType::Wheat1 => (221, 221, 221),
            BlockType::Wheat2 => (222, 222, 222), BlockType::Wheat3 => (223, 223, 223),
            BlockType::Wheat4 => (224, 224, 224), BlockType::Wheat5 => (225, 225, 225),
            BlockType::Wheat6 => (226, 226, 226), BlockType::Wheat7 => (227, 227, 227),
            BlockType::Cloud => (228, 228, 228),
            BlockType::Wheat => (80, 80, 80), BlockType::Bread => (81, 81, 81), BlockType::Apple => (82, 82, 82),
            BlockType::Porkchop => (83, 83, 83), BlockType::CookedPorkchop => (84, 84, 84),
            t if t.is_tool() => { let i = *t as u32; (i, i, i) }
            _ => (0, 0, 0),
        }
    }

pub fn get_display_name(&self) -> &str {
        match self {
            BlockType::Air => "Air", BlockType::Grass => "Grass", BlockType::Dirt => "Dirt",
            BlockType::Stone => "Stone", BlockType::Wood => "Oak Log", BlockType::Leaves => "Oak Leaves",
            BlockType::Snow => "Snow", BlockType::Sand => "Sand", BlockType::Bedrock => "Bedrock",
            BlockType::Water => "Water", BlockType::CoalOre => "Coal Ore", BlockType::IronOre => "Iron Ore",
            BlockType::GoldOre => "Gold Ore", BlockType::DiamondOre => "Diamond Ore",
            BlockType::RedstoneOre => "Redstone Ore", BlockType::LapisOre => "Lapis Ore",
            BlockType::Planks => "Planks", BlockType::Stick => "Stick", BlockType::Cobblestone => "Cobblestone",
            BlockType::IronIngot => "Iron Ingot", BlockType::GoldIngot => "Gold Ingot", BlockType::Diamond => "Diamond",
            BlockType::Torch => "Torch", BlockType::CraftingTable => "Crafting Table", BlockType::Furnace => "Furnace",
            BlockType::Chest | BlockType::ChestLeft | BlockType::ChestRight => "Chest",
            BlockType::Gravel => "Gravel", BlockType::Clay => "Clay", BlockType::Sandstone => "Sandstone",
            BlockType::Obsidian => "Obsidian", BlockType::Cactus => "Cactus", BlockType::Ice => "Ice",
            BlockType::LilyPad => "Lily Pad", BlockType::Mycelium => "Mycelium", BlockType::Vine => "Vine",
            BlockType::Rose => "Rose", BlockType::Dandelion => "Dandelion", BlockType::DeadBush => "Dead Bush",
            BlockType::TallGrass => "Tall Grass", BlockType::Sugarcane => "Sugarcane", BlockType::OakSapling => "Oak Sapling",
            BlockType::Glass => "Glass", BlockType::Bookshelf => "Bookshelf", BlockType::TNT => "TNT",
            BlockType::Pumpkin => "Pumpkin", BlockType::Melon => "Melon", BlockType::BrickBlock => "Bricks",
            BlockType::MossyCobble => "Mossy Cobblestone", BlockType::Lava => "Lava", BlockType::Fire => "Fire",
            BlockType::SpruceWood => "Spruce Log", BlockType::SpruceLeaves => "Spruce Leaves",
            BlockType::BirchWood => "Birch Log", BlockType::BirchLeaves => "Birch Leaves",
            BlockType::WheatSeeds => "Wheat Seeds", BlockType::Wheat => "Wheat", BlockType::Bread => "Bread",
            BlockType::Apple => "Apple", BlockType::Porkchop => "Raw Porkchop", BlockType::CookedPorkchop => "Cooked Porkchop",
            BlockType::BucketEmpty => "Empty Bucket", BlockType::BucketWater => "Water Bucket",
            BlockType::FarmlandDry => "Farmland", BlockType::FarmlandWet => "Hydrated Farmland",
            BlockType::GoldBlock => "Block of Gold", BlockType::IronBlock => "Block of Iron", BlockType::DiamondBlock => "Block of Diamond",
            t if t.is_tool() => match *t as u8 {
                21..=25 => match *t as u8 % 5 { 1=>"Wood Pickaxe", 2=>"Stone Pickaxe", 3=>"Iron Pickaxe", 4=>"Gold Pickaxe", 0=>"Diamond Pickaxe", _=>"Pickaxe" },
                26..=30 => match *t as u8 % 5 { 1=>"Wood Axe", 2=>"Stone Axe", 3=>"Iron Axe", 4=>"Gold Axe", 0=>"Diamond Axe", _=>"Axe" },
                31..=35 => match *t as u8 % 5 { 1=>"Wood Shovel", 2=>"Stone Shovel", 3=>"Iron Shovel", 4=>"Gold Shovel", 0=>"Diamond Shovel", _=>"Shovel" },
                36..=40 => match *t as u8 % 5 { 1=>"Wood Sword", 2=>"Stone Sword", 3=>"Iron Sword", 4=>"Gold Sword", 0=>"Diamond Sword", _=>"Sword" },
                41..=45 => match *t as u8 % 5 { 1=>"Wood Hoe", 2=>"Stone Hoe", 3=>"Iron Hoe", 4=>"Gold Hoe", 0=>"Diamond Hoe", _=>"Hoe" },
                _ => "Tool"
            },
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
            BlockType::Melon | BlockType::Pumpkin => 1.0,
            BlockType::Rose | BlockType::Dandelion | BlockType::TallGrass | BlockType::DeadBush | BlockType::OakSapling | BlockType::Sugarcane => 0.05, // Small value so they can be mined
            _ => 1.0,
        }
    }

    pub fn get_max_durability(&self) -> u16 {
        let u = *self as u8;
        if u >= 21 && u <= 25 { // Pickaxes
            match u % 5 { 1 => 60, 2 => 131, 3 => 250, 4 => 32, 0 => 1561, _ => 60 }
        } else if u >= 26 && u <= 30 { // Axes
            match u % 5 { 1 => 60, 2 => 131, 3 => 250, 4 => 32, 0 => 1561, _ => 60 }
        } else if u >= 31 && u <= 35 { // Shovels
            match u % 5 { 1 => 60, 2 => 131, 3 => 250, 4 => 32, 0 => 1561, _ => 60 }
        } else if u >= 36 && u <= 40 { // Swords
            match u % 5 { 1 => 60, 2 => 131, 3 => 250, 4 => 32, 0 => 1561, _ => 60 }
        } else { 0 }
    }
    
    pub fn get_best_tool_type(&self) -> &'static str {
        match self {
            BlockType::Stone | BlockType::Cobblestone | BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre | BlockType::DiamondOre | BlockType::Furnace => "pickaxe",
            BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Snow | BlockType::Gravel | BlockType::Clay => "shovel",
            BlockType::Wood | BlockType::Planks | BlockType::CraftingTable | BlockType::Leaves | BlockType::SpruceWood | BlockType::BirchWood | BlockType::Melon | BlockType::Pumpkin => "axe",
            _ => "none",
        }
    }
    
    
pub fn get_tool_class(&self) -> &'static str {
        let u = *self as u8;
        if u >= 21 && u <= 25 { "pickaxe" } 
        else if u >= 26 && u <= 30 { "axe" } 
        else if u >= 31 && u <= 35 { "shovel" } 
        else if u >= 36 && u <= 40 { "sword" } 
        else if u >= 41 && u <= 45 { "hoe" }
        else { "none" }
    }
    
    pub fn get_tool_speed(&self) -> f32 {
        let u = *self as u8;
        if u % 5 == 1 { 2.0 } else if u % 5 == 2 { 4.0 } else if u % 5 == 3 { 6.0 } else if u % 5 == 4 { 8.0 } else if u % 5 == 0 { 10.0 } else { 1.0 }
    }
}

pub struct Chunk { 
    pub blocks: Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]>,
    pub light: Box<[[[u8; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]>,
}
impl Chunk {
    pub fn new() -> Self { 
        Chunk { 
            blocks: Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]),
            light: Box::new([[[15u8; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]),
        } 
    }
    pub fn get_light(&self, x: usize, y: usize, z: usize) -> u8 { if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z { return 15; } self.light[x][y][z] }
    #[allow(dead_code)]
    pub fn set_light(&mut self, x: usize, y: usize, z: usize, val: u8) { if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z { self.light[x][y][z] = val; } }
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
        
        // 1. Base Terrain & Caves
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
                        let m = noise_gen.get_noise3d(wx as f64 * 0.008, 0.0, wz as f64 * 0.008);
                        let c = noise_gen.get_noise3d(wx as f64 * 0.02,  0.0, wz as f64 * 0.02);
                        let mountain = ((m.max(0.0)).powf(2.2) * 35.0) as i32;
                        let cliff = ((c.abs()).powf(3.0) * 18.0) as i32;
                        height = (height + mountain + cliff).clamp(1, (CHUNK_SIZE_Y as i32) - 3);
                        
                        let river_val = noise_gen.get_river_noise(wx, wz);
                        let mut is_river = false;
                        
                        // Rivers
                        if river_val.abs() < 0.08 {
                            let depth_factor = (0.08 - river_val.abs()) / 0.08;
                            height = (height as f32 * (1.0 - depth_factor as f32) + (WATER_LEVEL - 3) as f32 * depth_factor as f32) as i32;
                            is_river = true;
                        }
                        
                        let biome = noise_gen.get_biome(wx, wz, height);
                        
                        for y in 0..CHUNK_SIZE_Y {
                            let y_i32 = y as i32;
                            
                            // 3D Noise Caves (Swiss Cheese)
                            let cave_noise = noise_gen.get_noise3d(wx as f64 * 0.04, y as f64 * 0.04, wz as f64 * 0.04) 
                                           + noise_gen.get_noise3d(wx as f64 * 0.02, y as f64 * 0.02, wz as f64 * 0.02) * 0.5;
                            let is_cave = y_i32 < height && y_i32 > 5 && cave_noise > 0.6;
                            
                            if is_cave {
                                if y_i32 <= WATER_LEVEL { chunk.set_block(lx, y, lz, BlockType::Water); }
                                continue;
                            }

                            let mut block = BlockType::Air;
                            if y_i32 <= height {
                                if y_i32 == 0 { 
                                    block = BlockType::Bedrock; 
                                } else if y_i32 < 5 { 
                                    let mut rng = SimpleRng::new((wx^y_i32^wz) as u64);
                                    block = if rng.next_f32() < 0.5 { BlockType::Bedrock } else { BlockType::Stone }; 
                                } else if y_i32 < height - 3 {
                                    // ORE GENERATION
                                    let mut rng = SimpleRng::new((wx as u64).wrapping_mul(73856093) ^ (wz as u64).wrapping_mul(19349663) ^ (y_i32 as u64));
                                    let r = rng.next_f32();
                                    block = BlockType::Stone;
                                    if r < 0.012 { block = BlockType::CoalOre; } 
                                    else if r < 0.008 && y_i32 < 45 { block = BlockType::IronOre; } 
                                    else if r < 0.002 && y_i32 < 30 { block = BlockType::GoldOre; } 
                                    else if r < 0.003 && y_i32 < 16 { block = BlockType::RedstoneOre; }
                                    else if r < 0.002 && y_i32 < 16 { block = BlockType::LapisOre; }
                                    else if r < 0.001 && y_i32 < 12 { block = BlockType::DiamondOre; }
                                    else if r < 0.04 && y_i32 < 40 { block = BlockType::Gravel; }
                                    else if r < 0.04 && y_i32 < 40 { block = BlockType::Dirt; }
                                } else if y_i32 < height { 
                                    block = if is_river { BlockType::Sand } 
                                    else if biome == "desert" { BlockType::Sand } 
                                    else { BlockType::Dirt }; 
                                } else { 
                                    // Top Soil
                                    if height > 72 && biome != "desert" { 
                                        block = BlockType::Stone;
                                    } else if is_river { 
                                        if height < WATER_LEVEL { block = BlockType::Dirt; } else { block = BlockType::Sand; }
                                    } else if height >= 85 { 
                                        block = BlockType::Snow; 
                                    } else if biome == "desert" { 
                                        block = BlockType::Sand; 
                                    } else if height <= WATER_LEVEL + 1 { 
                                        block = BlockType::Sand; // Beaches
                                    } else { 
                                        block = BlockType::Grass; 
                                    }
                                }
                            } else if y_i32 <= WATER_LEVEL { 
                                block = BlockType::Water; 
                            }
                            
                            // Lava Pools Deep Down
                            if y_i32 < 10 && block == BlockType::Air && !is_river {
                                block = BlockType::Water; // Safety placeholder
                            }

                            if block != BlockType::Air { chunk.set_block(lx, y, lz, block); }
                        }
                    }
                }
                self.chunks.insert((cx, cz), chunk);
            }
        }
        
        // 2. Diabolical Decorators (Biome Specifics)
        for cx in -render_distance..=render_distance {
            for cz in -render_distance..=render_distance {
                let chunk_x_world = cx * (CHUNK_SIZE_X as i32); 
                let chunk_z_world = cz * (CHUNK_SIZE_Z as i32);
                for lx in 0..CHUNK_SIZE_X {
                    for lz in 0..CHUNK_SIZE_Z {
                        let wx = chunk_x_world + lx as i32; 
                        let wz = chunk_z_world + lz as i32;
                        let height = noise_gen.get_height(wx, wz);
                        
                        // Skip rivers for most decoration
                        if noise_gen.get_river_noise(wx, wz).abs() < 0.15 { 
                            if height < WATER_LEVEL - 1 {
                                let mut rng = SimpleRng::new((wx as u64).wrapping_mul(self.seed as u64).wrapping_add(wz as u64));
                                if rng.next_f32() < 0.2 { self.set_block_world(BlockPos{x:wx, y:height, z:wz}, BlockType::Clay); }
                            }
                            continue; 
                        }
                        
                        let biome = noise_gen.get_biome(wx, wz, height);
                        let mut rng = SimpleRng::new((wx as u64).wrapping_mul(self.seed as u64) ^ (wz as u64));
                        let r = rng.next_f32();

                        let surface_pos = BlockPos{x:wx, y:height, z:wz};
                        let above = BlockPos{x:wx, y:height+1, z:wz};
                        let ground = self.get_block(surface_pos);

                        if biome == "swamp" {
                             if ground == BlockType::Water && r < 0.05 { self.set_block_world(above, BlockType::LilyPad); }
                             if ground == BlockType::Grass && r < 0.02 {
                                 let h = 4 + (rng.next_f32() * 3.0) as i32;
                                 for i in 1..=h { self.set_block_world(BlockPos{x:wx, y:height+i, z:wz}, BlockType::Wood); }
                                 for ly in (height+h-2)..(height+h+2) {
                                     let rad = if ly > height+h { 1 } else { 3 };
                                     for dx in -rad..=rad { for dz in -rad..=rad {
                                         if (dx*dx + dz*dz) > rad*rad+2 { continue; }
                                         let lp = BlockPos{x:wx+dx, y:ly, z:wz+dz};
                                         if self.get_block(lp) == BlockType::Air { 
                                             self.set_block_world(lp, BlockType::Leaves);
                                             if rng.next_f32() < 0.2 && ly > height+2 {
                                                  let vlen = (rng.next_f32() * 3.0) as i32;
                                                  for k in 1..vlen { self.set_block_world(BlockPos{x:wx+dx, y:ly-k, z:wz+dz}, BlockType::Vine); }
                                             }
                                         }
                                     }}
                                 }
                             }
                        } else if biome == "taiga" {
                            if ground == BlockType::Grass && r < 0.02 {
                                let h = 6 + (rng.next_f32() * 4.0) as i32;
                                for i in 1..=h { self.set_block_world(BlockPos{x:wx, y:height+i, z:wz}, BlockType::SpruceWood); }
                                let mut rad: i32 = 2;
                                for ly in (height+3)..=(height+h) {
                                    for dx in -rad..=rad { for dz in -rad..=rad {
                                        if dx.abs() + dz.abs() > rad { continue; }
                                        let lp = BlockPos{x:wx+dx, y:ly, z:wz+dz};
                                        if self.get_block(lp) == BlockType::Air { self.set_block_world(lp, BlockType::SpruceLeaves); }
                                    }}
                                    if ly % 2 == 0 { rad = (rad - 1).max(0); }
                                }
                                self.set_block_world(BlockPos{x:wx, y:height+h+1, z:wz}, BlockType::SpruceLeaves);
                            }
                        } else if biome == "desert" {
                             if ground == BlockType::Sand {
                                 if r < 0.01 {
                                     let h = 1 + (rng.next_f32() * 3.0) as i32;
                                     for i in 0..h { self.set_block_world(BlockPos{x:wx, y:height+1+i, z:wz}, BlockType::Cactus); }
                                 } else if r < 0.02 { self.set_block_world(above, BlockType::DeadBush); }
                             }
                        } else if biome == "ice_plains" {
                             if height == WATER_LEVEL { self.set_block_world(surface_pos, BlockType::Ice); }
                        } else {
                            if ground == BlockType::Grass {
                                if biome == "forest" && r < 0.01 {
                                    let tree_h = 4 + (rng.next_f32() * 2.0) as i32;
                                    for i in 1..=tree_h { self.set_block_world(BlockPos{x:wx, y:height+i, z:wz}, BlockType::Wood); }
                                    for ly in (height+tree_h-2)..(height+tree_h+2) {
                                        let rad = if ly > height+tree_h { 1 } else { 2 };
                                        for dx in -rad..=rad { for dz in -rad..=rad {
                                            if (dx*dx + dz*dz) > rad*rad+1 { continue; }
                                            let lp = BlockPos{x:wx+dx, y:ly, z:wz+dz};
                                            if self.get_block(lp) == BlockType::Air { self.set_block_world(lp, BlockType::Leaves); }
                                        }}
                                    }
                                }
                                if r < 0.01 { self.set_block_world(above, BlockType::Rose); }
                                else if r < 0.02 { self.set_block_world(above, BlockType::Dandelion); }
                                else if r < 0.05 && biome == "plains" { self.set_block_world(above, BlockType::TallGrass); }
                                else if r < 0.002 { self.set_block_world(above, BlockType::Pumpkin); }
                                else if r > 0.998 { self.set_block_world(above, BlockType::Melon); }
                            }
                             if (ground == BlockType::Grass || ground == BlockType::Sand) && r < 0.05 {
                                 let mut near_water = false;
                                 for (dx, dz) in &[(1,0),(-1,0),(0,1),(0,-1)] {
                                     if self.get_block(BlockPos{x:wx+dx, y:height, z:wz+dz}).is_water() { near_water = true; break; }
                                 }
                                 if near_water {
                                     let h = 1 + (rng.next_f32() * 3.0) as i32;
                                     for i in 0..h { self.set_block_world(BlockPos{x:wx, y:height+1+i, z:wz}, BlockType::Sugarcane); }
                                 }
                             }
                        }
                    }
                }
            }
        }
    }
pub fn get_light_world(&self, pos: BlockPos) -> u8 {
        let cx = pos.x.div_euclid(CHUNK_SIZE_X as i32); let cz = pos.z.div_euclid(CHUNK_SIZE_Z as i32);
        let lx = pos.x.rem_euclid(CHUNK_SIZE_X as i32) as usize; let lz = pos.z.rem_euclid(CHUNK_SIZE_Z as i32) as usize;
        if pos.y < 0 || pos.y >= CHUNK_SIZE_Y as i32 { return 15; }
        if let Some(chunk) = self.chunks.get(&(cx, cz)) { chunk.get_light(lx, pos.y as usize, lz) } else { 15 }
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
// --- PHYSICS & LOGIC ---
#[allow(dead_code)]
    pub fn update_block_physics(&mut self, pos: BlockPos) -> Vec<(i32, i32)> {
        let mut affected = Vec::new();
        let b = self.get_block(pos);
        
        // 1. Gravity (Sand/Gravel)
        if matches!(b, BlockType::Sand | BlockType::Gravel) {
            let below_pos = BlockPos { x: pos.x, y: pos.y - 1, z: pos.z };
            let below = self.get_block(below_pos);
            if below == BlockType::Air || below.is_water() || below == BlockType::Lava {
                self.set_block_world(pos, BlockType::Air);
                self.set_block_world(below_pos, b);
                affected.extend(self.get_affected_chunks(pos));
                affected.extend(self.get_affected_chunks(below_pos));
                // Recursive update
                affected.extend(self.update_block_physics(below_pos));
                // Update neighbors of old pos
                for (dx, dy, dz) in &[(0,1,0), (0,-1,0), (1,0,0), (-1,0,0), (0,0,1), (0,0,-1)] {
                     affected.extend(self.update_block_physics(BlockPos{x:pos.x+dx, y:pos.y+dy, z:pos.z+dz}));
                }
            }
        }

        // 2. Fluid Interaction (Obsidian/Cobble Gen)
        if b == BlockType::Lava {
             for (dx, dy, dz) in &[(1,0,0), (-1,0,0), (0,0,1), (0,0,-1), (0,1,0)] {
                 let n_pos = BlockPos{x:pos.x+dx, y:pos.y+dy, z:pos.z+dz};
                 if self.get_block(n_pos).is_water() {
                     self.set_block_world(pos, BlockType::Obsidian);
                     affected.extend(self.get_affected_chunks(pos));
                     break;
                 }
             }
        } else if b.is_water() {
             for (dx, dy, dz) in &[(1,0,0), (-1,0,0), (0,0,1), (0,0,-1), (0,1,0)] {
                 let n_pos = BlockPos{x:pos.x+dx, y:pos.y+dy, z:pos.z+dz};
                 if self.get_block(n_pos) == BlockType::Lava {
                     self.set_block_world(n_pos, BlockType::Obsidian);
                     affected.extend(self.get_affected_chunks(n_pos));
                 }
             }
        }
        affected.sort(); affected.dedup();
        affected
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
             let drop_item = match block_type { BlockType::Stone => BlockType::Cobblestone, BlockType::CoalOre => BlockType::Coal, BlockType::IronOre => BlockType::IronOre, BlockType::DiamondOre => BlockType::DiamondItem, BlockType::Grass => BlockType::Dirt, _ => block_type };
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
                                let new_blk = if lvl > 1 { BlockType::Water } else { BlockType::Air };
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

        // Gravity
        entity.velocity.y -= 25.0 * dt;

        // Physics
        let next_pos = entity.position + entity.velocity * dt;

        // Helper to check solids
        let solid_at = |x: f32, y: f32, z: f32| -> bool { 
            self.get_block(BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 }).is_solid() 
        };

        // X Collision
        let try_x = Vec3::new(next_pos.x, entity.position.y, entity.position.z);
        if !solid_at(try_x.x, try_x.y, try_x.z) { entity.position.x = try_x.x; }

        // Z Collision
        let try_z = Vec3::new(entity.position.x, entity.position.y, next_pos.z);
        if !solid_at(try_z.x, try_z.y, try_z.z) { entity.position.z = try_z.z; }

        // Y Collision (Ground Snap)
        let try_y = Vec3::new(entity.position.x, next_pos.y, entity.position.z);
        let feet_y = try_y.y - 0.1;
        if entity.velocity.y <= 0.0 && solid_at(try_y.x, feet_y, try_y.z) {
            entity.velocity.y = 0.0; entity.velocity.x *= 0.85; entity.velocity.z *= 0.85;
            entity.position.y = feet_y.floor() + 1.1; // Snap up
        } else if !solid_at(try_y.x, try_y.y, try_y.z) {
            entity.position.y = try_y.y;
        }

        // Pickup Logic
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