use std::collections::{HashMap, VecDeque, HashSet};
use crate::noise_gen::NoiseGenerator;
use crate::player::Player;
use glam::Vec3;
use serde::{Serialize, Deserialize};

pub struct SimpleRng { pub state: u64 }
impl SimpleRng {
    pub fn new(seed: u64) -> Self { Self { state: seed } }
    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) ^ self.state) as u32 as f32 / u32::MAX as f32
    }
    fn gen_range(&mut self, min: f32, max: f32) -> f32 { min + (max - min) * self.next_f32() }
}

pub const CHUNK_SIZE_X: usize = 16;
pub const CHUNK_SIZE_Z: usize = 16;
pub const CHUNK_SIZE_Y: usize = 16; // DIABOLICAL VERTICAL SUBDIVISION
pub const WORLD_HEIGHT: i32 = 128;
pub const WATER_LEVEL: i32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    Air = 0, Grass = 1, Dirt = 2, Stone = 3, Wood = 4, Leaves = 5, Snow = 6, Sand = 7, Bedrock = 8, Water = 9,
    CoalOre = 10, IronOre = 11, GoldOre = 12, DiamondOre = 13, RedstoneOre = 125, LapisOre = 126,
Planks = 14, Stick = 15, Cobblestone = 16, IronIngot = 17, GoldIngot = 18, Diamond = 19, Torch = 20,
    SprucePlanks = 170, BirchPlanks = 171,
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
    pub fn is_liquid(&self) -> bool { matches!(self, BlockType::Water | BlockType::Lava) }
    pub fn get_texture_top(&self) -> u32 { self.get_texture_indices().0 }
    pub fn get_texture_bottom(&self) -> u32 { self.get_texture_indices().1 }
    pub fn get_texture_side(&self) -> u32 { self.get_texture_indices().2 }
pub fn is_transparent(&self) -> bool { 
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Leaves | BlockType::SpruceLeaves | BlockType::BirchLeaves | 
                       BlockType::Torch | BlockType::Fire | BlockType::Glass | BlockType::Rose | BlockType::Dandelion | 
                       BlockType::DeadBush | BlockType::TallGrass | BlockType::OakSapling | BlockType::Sugarcane | 
                       BlockType::Ice | BlockType::LilyPad | BlockType::Vine |
                       BlockType::Wheat0 | BlockType::Wheat1 | BlockType::Wheat2 | BlockType::Wheat3 | 
                       BlockType::Wheat4 | BlockType::Wheat5 | BlockType::Wheat6 | BlockType::Wheat7) 
    }

    pub fn is_cross_model(&self) -> bool {
        matches!(self, BlockType::DeadBush | BlockType::TallGrass | BlockType::OakSapling | BlockType::Sugarcane | 
                       BlockType::Rose | BlockType::Dandelion | BlockType::Wheat | BlockType::Wheat0 | 
                       BlockType::Wheat1 | BlockType::Wheat2 | BlockType::Wheat3 | BlockType::Wheat4 | 
                       BlockType::Wheat5 | BlockType::Wheat6 | BlockType::Wheat7)
    }

    pub fn is_water(&self) -> bool { matches!(self, BlockType::Water) }

    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Fire | 
                       BlockType::Rose | BlockType::Dandelion | BlockType::DeadBush | BlockType::TallGrass | 
                       BlockType::OakSapling | BlockType::Sugarcane | BlockType::LilyPad | BlockType::Vine | 
                       BlockType::Wheat | BlockType::Wheat0 | BlockType::Wheat1 | BlockType::Wheat2 | 
                       BlockType::Wheat3 | BlockType::Wheat4 | BlockType::Wheat5 | BlockType::Wheat6 | BlockType::Wheat7)
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

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]>,
    pub light: Box<[[[u8; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]>,
    pub is_empty: bool,
    pub mesh_dirty: bool,
}
impl Chunk {
    pub fn new() -> Self { 
        Chunk { 
            blocks: Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]),
            light: Box::new([[[15u8; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]),
            is_empty: true,
            mesh_dirty: true,
        } 
    }
    pub fn get_light(&self, x: usize, y: usize, z: usize) -> u8 { if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z { return 15; } self.light[x][y][z] }
#[allow(dead_code)]
    pub fn set_light(&mut self, x: usize, y: usize, z: usize, val: u8) { if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z { self.light[x][y][z] = val; } }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType { if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z { return BlockType::Air; } self.blocks[x][y][z] }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) { if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z { self.blocks[x][y][z] = block; } }
}
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct ItemEntity { pub position: Vec3, pub velocity: Vec3, pub item_type: BlockType, pub count: u8, pub pickup_delay: f32, pub lifetime: f32, pub rotation: f32, pub bob_offset: f32 }
#[derive(Clone, Copy)]
pub struct RemotePlayer { pub id: u32, pub position: Vec3, pub rotation: f32 }

#[derive(Clone)]
pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
    pub entities: Vec<ItemEntity>,
    pub mesh_dirty: bool,
    pub dirty_chunks: HashSet<(i32, i32, i32)>, // NEW: Priority mesh update queue
    pub remote_players: Vec<RemotePlayer>,
    pub seed: u32,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let world = World { 
            chunks: HashMap::new(), 
            entities: Vec::new(), 
            mesh_dirty: true,
            dirty_chunks: HashSet::new(),
            remote_players: Vec::new(), 
            seed 
        };
        // DIABOLICAL STARTUP: Do NOT generate terrain here.
        // The main loop will handle this during the Loading state to keep the OS responsive.
        world
    }

pub fn generate_one_chunk_around(&mut self, cx: i32, _cy: i32, cz: i32, radius: i32) -> Option<(i32, i32, i32)> {
        let noise_gen = NoiseGenerator::new(self.seed);
        for r in 0..=radius {
            for x in -r..=r {
                for z in -r..=r {
                    let tcx = cx + x;
                    let tcz = cz + z;
                    // Check all vertical chunks for this column
                    for y in 0..(WORLD_HEIGHT / 16 as i32) {
                        if !self.chunks.contains_key(&(tcx, y, tcz)) {
                            self.generate_single_chunk(tcx, y, tcz, &noise_gen);
                            return Some((tcx, y, tcz));
                        }
                    }
                }
            }
        }
        None
    }

pub fn generate_terrain_around(&mut self, cx: i32, cz: i32, radius: i32) -> Vec<(i32, i32, i32)> {
        let mut newly_generated = Vec::new();
        let noise_gen = NoiseGenerator::new(self.seed);
        for x in -radius..=radius {
            for z in -radius..=radius {
                for y in 0..(WORLD_HEIGHT / 16 as i32) {
                    let (tcx, tcy, tcz) = (cx + x, y, cz + z);
                    if !self.chunks.contains_key(&(tcx, tcy, tcz)) {
                        self.generate_single_chunk(tcx, tcy, tcz, &noise_gen);
                        newly_generated.push((tcx, tcy, tcz));
                    }
                }
            }
        }
        newly_generated
    }

    pub fn bootstrap_terrain_step(&mut self, step: i32) -> bool {
        let radius = 6;
        let side = radius * 2 + 1;
        if step >= side * side { return true; }
        
        let x = (step % side) - radius;
        let z = (step / side) - radius;
        let noise_gen = NoiseGenerator::new(self.seed);
        
        log::info!("[WORLD] Generating Column: X:{:<3} Z:{:<3} (Step {}/{})", x, z, step, side * side);

        for y in 0..(WORLD_HEIGHT / 16) {
            if !self.chunks.contains_key(&(x, y, z)) {
                self.generate_single_chunk(x, y, z, &noise_gen);
            }
        }
        false
    }
pub fn _update_occlusion(&mut self, _px: i32, _py: i32, _pz: i32) {
        // ROOT FIX: Occlusion is now handled by the Frustum Culler in the Renderer.
        // We do nothing here to keep the World structure lightweight.
    }

    fn generate_single_chunk(&mut self, cx: i32, cy: i32, cz: i32, noise_gen: &NoiseGenerator) {
        let mut chunk = Chunk::new();
        let chunk_x_world = cx * 16;
        let chunk_y_world = cy * 16;
        let chunk_z_world = cz * 16;
        let mut rng = SimpleRng::new((cx as u64).wrapping_mul(self.seed as u64) ^ (cy as u64) ^ (cz as u64));
        let mut tree_map: HashSet<(i32, i32)> = HashSet::new();

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = chunk_x_world + lx as i32;
                let wz = chunk_z_world + lz as i32;
                let (cont, eros, weird, temp) = noise_gen.get_height_params(wx, wz);
                let humid = noise_gen.get_noise_octaves(wx as f64 * 0.01, 123.0, wz as f64 * 0.01, 3) as f32;
                
                for ly in 0..CHUNK_SIZE_Y {
                    let y = chunk_y_world + ly as i32;
                    let density = noise_gen.get_density(wx, y, wz, cont, eros, weird);
                    let mut block = BlockType::Air;

                    if density > 0.0 {
                        let n1 = noise_gen.get_noise3d(wx as f64 * 0.06, y as f64 * 0.06, wz as f64 * 0.06);
                        let n2 = noise_gen.get_noise3d(wx as f64 * 0.06, y as f64 * 0.06 + 100.0, wz as f64 * 0.06);
                        if n1.abs() < 0.05 && n2.abs() < 0.05 && y > 6 { 
                            block = if (y as i32) <= WATER_LEVEL { BlockType::Water } else { BlockType::Air };
                        } else {
                            if noise_gen.get_density(wx, (y + 1) as i32, wz, cont, eros, weird) <= 0.0 {
                                block = match noise_gen.get_biome(cont, eros, temp, humid, y as i32) {
                                    "desert" => BlockType::Sand,
                                    "ice_plains" | "ice_ocean" => BlockType::Snow,
                                    "ocean" | "badlands" => BlockType::Sand,
                                    "peaks" => BlockType::Stone,
                                    _ => BlockType::Grass,
                                };
                            } else {
                                block = if y < 5 { BlockType::Bedrock } else if (y as i32) < (62 + (cont * 12.0) as i32) { BlockType::Stone } else { BlockType::Dirt };
                            }
                        }
                    } else if (y as i32) <= WATER_LEVEL { block = BlockType::Water; }

                    if block != BlockType::Air { 
                        chunk.set_block(lx, ly, lz, block); 
                        chunk.is_empty = false;
                    }
                }

                let h_world = noise_gen.get_height(wx, wz);
                if h_world >= chunk_y_world && h_world < chunk_y_world + 16 {
                    let ly = (h_world - chunk_y_world) as usize;
                    let biome = noise_gen.get_biome(cont, eros, temp, humid, h_world);
                    let r = rng.next_f32();
                    let ground_block = chunk.get_block(lx, ly, lz);

                    if matches!(ground_block, BlockType::Grass | BlockType::Dirt | BlockType::Sand | BlockType::Snow) {
                        if noise_gen.get_density(wx, h_world + 5, wz, cont, eros, weird) < 0.0 {
                            let mut too_close = false;
                            for dx in -4..=4 { 
                                for dz in -4..=4 { 
                                    if tree_map.contains(&(lx as i32 + dx, lz as i32 + dz)) { too_close = true; break; } 
                                } 
                                if too_close { break; } 
                            }

                            if !too_close {
                                if (biome == "forest" || biome == "jungle") && r < 0.05 {
                                    tree_map.insert((lx as i32, lz as i32));
                                    let tree_h = 5 + (rng.next_f32() * 2.0) as i32;
                                    // PLACE LOGS
                                    for i in 1..=tree_h { 
                                        let ty = h_world + i;
                                        if ty >= chunk_y_world && ty < chunk_y_world + 16 {
                                            chunk.set_block(lx, (ty - chunk_y_world) as usize, lz, BlockType::Wood); 
                                        }
                                    }
                                    // PLACE LEAF CANOPY
                                    for dy in (tree_h - 2)..=(tree_h + 1) {
                                        for dx in -2..=2 {
                                            for dz in -2..=2 {
                                                if dx*dx + dz*dz > 4 { continue; } // Rounded canopy
                                                let ty = h_world + dy;
                                                let tlx = lx as i32 + dx;
                                                let tlz = lz as i32 + dz;
                                                if ty >= chunk_y_world && ty < chunk_y_world + 16 && tlx >= 0 && tlx < 16 && tlz >= 0 && tlz < 16 {
                                                    let cur = chunk.get_block(tlx as usize, (ty - chunk_y_world) as usize, tlz as usize);
                                                    if cur == BlockType::Air {
                                                        chunk.set_block(tlx as usize, (ty - chunk_y_world) as usize, tlz as usize, BlockType::Leaves);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if r < 0.02 {
                                    if ly + 1 < 16 {
                                        chunk.set_block(lx, ly + 1, lz, if biome == "desert" { BlockType::DeadBush } else { BlockType::Rose });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        self.chunks.insert((cx, cy, cz), chunk);
    }

    fn _get_height_at_in_chunk(&self, chunk: &Chunk, lx: usize, lz: usize) -> i32 {
        for y in (0..CHUNK_SIZE_Y as i32).rev() {
            if chunk.get_block(lx, y as usize, lz).is_solid() { return y; }
        }
        0
    }
#[allow(dead_code)]
    fn generate_terrain(&mut self) {
        // Consolidated terrain generation to prevent logic duplication and tree-overlap
        self.generate_terrain_around(0, 0, 6);
    }
pub fn get_light_world(&self, pos: BlockPos) -> u8 {
        let cx = pos.x.div_euclid(16); let cy = pos.y.div_euclid(16); let cz = pos.z.div_euclid(16);
        let lx = pos.x.rem_euclid(16) as usize; let ly = pos.y.rem_euclid(16) as usize; let lz = pos.z.rem_euclid(16) as usize;
        if pos.y < 0 || pos.y >= WORLD_HEIGHT { return 15; }
        if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) { chunk.get_light(lx, ly, lz) } else { 15 }
    }
    pub fn get_height_at(&self, x: i32, z: i32) -> i32 {
        for cy in (0..(WORLD_HEIGHT/16)).rev() {
            let cx = x.div_euclid(16); let cz = z.div_euclid(16);
            if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) {
                let lx = x.rem_euclid(16) as usize;
                let lz = z.rem_euclid(16) as usize;
                for ly in (0..16).rev() {
                    if chunk.get_block(lx, ly, lz).is_solid() { return cy * 16 + ly as i32; }
                }
            }
        }
        0
    }
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        let cx = pos.x.div_euclid(16); let cy = pos.y.div_euclid(16); let cz = pos.z.div_euclid(16);
        let lx = pos.x.rem_euclid(16) as usize; let ly = pos.y.rem_euclid(16) as usize; let lz = pos.z.rem_euclid(16) as usize;
        if pos.y < 0 || pos.y >= WORLD_HEIGHT { return BlockType::Air; }
        if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) { chunk.get_block(lx, ly, lz) } else { BlockType::Air }
    }
pub fn set_block_world(&mut self, pos: BlockPos, block: BlockType) {
        let cx = pos.x.div_euclid(16); let cy = pos.y.div_euclid(16); let cz = pos.z.div_euclid(16);
        let lx = pos.x.rem_euclid(16) as usize; let ly = pos.y.rem_euclid(16) as usize; let lz = pos.z.rem_euclid(16) as usize;
        if cy < 0 || cy >= 8 { return; }
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy, cz)) { 
            chunk.set_block(lx, ly, lz, block); 
            chunk.mesh_dirty = true;
            if block != BlockType::Air { chunk.is_empty = false; }
        }
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
    pub fn update_block_physics(&mut self, pos: BlockPos) -> Vec<(i32, i32, i32)> {
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
        affected.sort_unstable(); affected.dedup();
        affected
    }
    pub fn get_affected_chunks(&self, pos: BlockPos) -> Vec<(i32, i32, i32)> {
        // DIABOLICAL 3D RADIUS: Ghost blocks occur because boundary neighbors don't know they need to redraw.
        let cx = pos.x.div_euclid(16);
        let cy = pos.y.div_euclid(16);
        let cz = pos.z.div_euclid(16);
        
        let mut affected = Vec::new();
        
        // ROOT CAUSE FIX: We must update a 3x3x3 cube of chunks around the change.
        // Greedy meshing can stretch faces across chunk boundaries; if a block on the edge
        // is removed, the neighbor chunk's mesh MUST be invalidated.
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let target_y = cy + dy;
                    if target_y >= 0 && target_y < (WORLD_HEIGHT / 16) {
                        affected.push((cx + dx, target_y, cz + dz));
                    }
                }
            }
        }
        
        affected.sort_unstable();
        affected.dedup();
        affected
    }

    pub fn break_block(&mut self, pos: BlockPos) -> Vec<(i32, i32, i32)> {
        let block_type = self.get_block(pos);
        if block_type != BlockType::Air && block_type != BlockType::Bedrock && !block_type.is_water() {
            self.mesh_dirty = true;
            let mut rng = SimpleRng::new(pos.x as u64 ^ pos.z as u64 ^ pos.y as u64);
            let velocity = Vec3::new(rng.gen_range(-2.0, 2.0), 4.0, rng.gen_range(-2.0, 2.0));
            let drop_item = match block_type { 
                BlockType::Stone => BlockType::Cobblestone, 
                BlockType::CoalOre => BlockType::Coal, 
                BlockType::Grass => BlockType::Dirt, 
                _ => block_type 
            };
            self.entities.push(ItemEntity { 
                position: Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5), 
                velocity, 
                item_type: drop_item, 
                count: 1, 
                pickup_delay: 1.0, 
                lifetime: 300.0, 
                rotation: 0.0, 
                bob_offset: rng.next_f32() * 10.0 
            });
        }
        
        self.set_block_world(pos, BlockType::Air);
        
        let mut affected = self.trigger_water_update(pos);
        affected.extend(self.get_affected_chunks(pos));
        affected.sort_unstable();
        affected.dedup();
        
        for &(cx, cy, cz) in &affected {
            if let Some(chunk) = self.chunks.get_mut(&(cx, cy, cz)) {
                chunk.mesh_dirty = true;
                self.dirty_chunks.insert((cx, cy, cz)); // PRIORITY UPDATE
            }
        }
        self.mesh_dirty = true;
        affected
    }
    pub fn place_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<(i32, i32, i32)> { 
        self.set_block_world(pos, block); 
        let mut affected = self.trigger_water_update(pos);
        affected.extend(self.get_affected_chunks(pos));
        affected.sort_unstable();
        affected.dedup();
        
        for &(cx, cy, cz) in &affected {
            if let Some(chunk) = self.chunks.get_mut(&(cx, cy, cz)) {
                chunk.mesh_dirty = true;
                self.dirty_chunks.insert((cx, cy, cz)); // PRIORITY UPDATE
            }
        }
        self.mesh_dirty = true;
        affected
    }
    fn trigger_water_update(&mut self, start_pos: BlockPos) -> Vec<(i32, i32, i32)> {
        let mut updates = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_pos);
        let mut visited = HashSet::new();
        let mut steps = 0;

        while let Some(pos) = queue.pop_front() {
            if steps > 15 { break; } // Hard limit for performance
            if !visited.insert(pos) { continue; }
            steps += 1;

            let current = self.get_block(pos);
            let cx = pos.x.div_euclid(16);
            let cy = pos.y.div_euclid(16);
            let cz = pos.z.div_euclid(16);
            updates.push((cx, cy, cz));

            if current.is_water() {
                let below = BlockPos { x: pos.x, y: pos.y - 1, z: pos.z };
                if self.get_block(below) == BlockType::Air {
                    self.set_block_world(below, BlockType::Water);
                    queue.push_back(below);
                }
            }
        }
        updates.sort_unstable();
        updates.dedup();
        updates
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