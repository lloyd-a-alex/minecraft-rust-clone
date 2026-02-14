use std::collections::{HashMap, VecDeque, HashSet};
use crate::resources::NoiseGenerator;
use glam::Vec3;

pub struct SimpleRng { pub state: u64 }
impl SimpleRng {
    pub fn new(seed: u64) -> Self { Self { state: seed } }
    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) ^ self.state) as u32 as f32 / u32::MAX as f32
    }
    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 { min + (max - min) * self.next_f32() }
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
    CoalOre = 10, IronOre = 11, GoldOre = 12, DiamondOre = 13, RedstoneOre = 140, LapisOre = 141,
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
    GoldBlock = 180, IronBlock = 181, DiamondBlock = 182,
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
    RottenFlesh = 116, Carrot = 117, Potato = 118, Arrow = 119, Bone = 130, Bow = 131,
    Gunpowder = 132, String = 133, SpiderEye = 134, Leather = 135, Beef = 136, CookedBeef = 137,
    Wool = 138, Mutton = 139, CookedMutton = 150, Feather = 151, Chicken = 152, CookedChicken = 153,
    EnderPearl = 144, GlowstoneDust = 145, Redstone = 146,
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
            BlockType::RottenFlesh => (85, 85, 85), BlockType::Carrot => (86, 86, 86), BlockType::Potato => (87, 87, 87),
            BlockType::Arrow => (88, 88, 88), BlockType::Bone => (89, 89, 89), BlockType::Bow => (90, 90, 90),
            BlockType::Gunpowder => (91, 91, 91), BlockType::String => (92, 92, 92), BlockType::SpiderEye => (93, 93, 93),
            BlockType::Leather => (94, 94, 94), BlockType::Beef => (95, 95, 95), BlockType::CookedBeef => (96, 96, 96),
            BlockType::Wool => (97, 97, 97), BlockType::Mutton => (98, 98, 98), BlockType::CookedMutton => (99, 99, 99),
            BlockType::Feather => (100, 100, 100), BlockType::Chicken => (101, 101, 101), BlockType::CookedChicken => (102, 102, 102),
            BlockType::EnderPearl => (103, 103, 103), BlockType::GlowstoneDust => (104, 104, 104), BlockType::Redstone => (105, 105, 105),
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

    pub fn get_step_sound_category(&self) -> &'static str {
        match self {
            BlockType::Grass | BlockType::Mycelium | BlockType::LilyPad => "grass",
            BlockType::Dirt | BlockType::FarmlandDry | BlockType::FarmlandWet | BlockType::Gravel | BlockType::Clay => "gravel",
            BlockType::Stone | BlockType::Cobblestone | BlockType::MossyCobble | BlockType::BrickBlock | 
            BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre | BlockType::DiamondOre | 
            BlockType::RedstoneOre | BlockType::LapisOre | BlockType::Furnace | BlockType::FurnaceActive |
            BlockType::Obsidian => "stone",
            BlockType::Wood | BlockType::Planks | BlockType::SpruceWood | BlockType::SprucePlanks | 
            BlockType::BirchWood | BlockType::BirchPlanks | BlockType::Chest | BlockType::ChestLeft | 
            BlockType::ChestRight | BlockType::CraftingTable | BlockType::Bookshelf | BlockType::OakSapling => "wood",
            BlockType::Leaves | BlockType::SpruceLeaves | BlockType::BirchLeaves | BlockType::Vine | 
            BlockType::Cactus | BlockType::Rose | BlockType::Dandelion | BlockType::TallGrass | BlockType::Sugarcane |
            BlockType::Wheat | BlockType::Wheat0 | BlockType::Wheat1 | BlockType::Wheat2 | BlockType::Wheat3 | 
            BlockType::Wheat4 | BlockType::Wheat5 | BlockType::Wheat6 | BlockType::Wheat7 => "leaves",
            BlockType::Sand | BlockType::Sandstone => "sand",
            BlockType::Snow | BlockType::Ice => "snow",
            BlockType::Glass => "glass",
            BlockType::Bedrock => "bedrock",
            BlockType::IronBlock | BlockType::GoldBlock | BlockType::DiamondBlock | BlockType::TNT => "metal",
            BlockType::Water | BlockType::Lava => "water",
            _ => "stone",
        }
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
                
                // DIABOLICAL VOLUMETRIC SAMPLING
                for ly in 0..16 {
                    let y_world = chunk_y_world + ly as i32;
                    let mut block = BlockType::Air;
                    let density = noise_gen.get_density(wx, y_world, wz, cont, eros, weird);

                    if density > 0.0 {
                        let density_above = noise_gen.get_density(wx, y_world + 1, wz, cont, eros, weird);
                        let is_surface = density_above <= 0.0 && y_world > WATER_LEVEL;
                        let biome = noise_gen.get_biome(cont, eros, temp, humid, y_world);
                        
                        block = if is_surface {
                            if biome == "desert" || biome == "ocean" { BlockType::Sand }
                            else if biome == "ice_plains" { BlockType::Snow }
                            else { BlockType::Grass }
                        } else if density < 0.15 && y_world > WATER_LEVEL - 5 {
                            if biome == "desert" { BlockType::Sand } else { BlockType::Dirt }
                        } else {
                            BlockType::Stone
                        };
                    } else if y_world <= WATER_LEVEL {
                        block = BlockType::Water;
                    }

                    if y_world < 2 { block = BlockType::Bedrock; }

                    if block != BlockType::Air {
                        if block == BlockType::Stone {
                            let ore_rng = (wx as u64).wrapping_mul(31234) ^ (y_world as u64).wrapping_mul(7123) ^ (wz as u64).wrapping_mul(1234);
                            let ore_chance = (ore_rng % 1000) as f32 / 10.0;
                            if y_world < 16 && ore_chance < 0.2 { block = BlockType::DiamondOre; }
                            else if y_world < 32 && ore_chance < 0.5 { block = BlockType::GoldOre; }
                            else if y_world < 48 && ore_chance < 0.8 { block = BlockType::LapisOre; }
                            else if y_world < 64 && ore_chance < 1.2 { block = BlockType::RedstoneOre; }
                            else if y_world < 64 && ore_chance < 2.5 { block = BlockType::IronOre; }
                            else if ore_chance < 4.0 { block = BlockType::CoalOre; }
                        }
                        chunk.set_block(lx, ly, lz, block);
                        chunk.is_empty = false;
                    }
                }

                // DIABOLICAL SURFACE SCAN
                for ly in 0..CHUNK_SIZE_Y {
                    let y_world = chunk_y_world + ly as i32;
                    let density = noise_gen.get_density(wx, y_world, wz, cont, eros, weird);
                    let density_above = noise_gen.get_density(wx, y_world + 1, wz, cont, eros, weird);
                    
                    if density > 0.0 && density_above <= 0.0 && y_world > WATER_LEVEL {
                        let biome = noise_gen.get_biome(cont, eros, temp, humid, y_world);
                        let r = rng.next_f32();
                        let ground_block = chunk.get_block(lx, ly, lz);

                        if matches!(ground_block, BlockType::Grass | BlockType::Dirt | BlockType::Sand | BlockType::Snow) {
                            if noise_gen.get_density(wx, y_world + 5, wz, cont, eros, weird) < 0.0 {
                                let mut too_close = false;
                                for dx in -4..=4 { 
                                    for dz in -4..=4 { 
                                        if tree_map.contains(&(lx as i32 + dx, lz as i32 + dz)) { too_close = true; break; } 
                                    } 
                                    if too_close { break; } 
                                }

                                if !too_close {
                                    if (biome == "forest" || biome == "jungle") && r < 0.05 && y_world > 64 {
                                        tree_map.insert((lx as i32, lz as i32));
                                        let tree_h = 5 + (rng.next_f32() * 3.0) as i32;
                                        for i in 1..=tree_h { 
                                            let ty = y_world + i;
                                            if ty >= chunk_y_world && ty < chunk_y_world + 16 {
                                                chunk.set_block(lx, (ty - chunk_y_world) as usize, lz, BlockType::Wood); 
                                            }
                                        }
                                        for dy in (tree_h - 2)..=(tree_h + 1) {
                                            let radius: i32 = if dy > tree_h { 0 } else if dy == tree_h { 1 } else { 2 };
                                            for dx in -radius..=radius {
                                                for dz in -radius..=radius {
                                                    if dx.abs() + dz.abs() > radius + 1 { continue; }
                                                    let ty = y_world + dy;
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
        let cx = x.div_euclid(16);
        let cz = z.div_euclid(16);
        let lx = x.rem_euclid(16) as usize;
        let lz = z.rem_euclid(16) as usize;

        for cy in (0..8).rev() { // 8 vertical chunks = 128 height
            if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) {
                for ly in (0..16).rev() {
                    let block = chunk.get_block(lx, ly, lz);
                    if block.is_solid() || block.is_water() { 
                        return cy * 16 + ly as i32; 
                    }
                }
            }
        }
        64 // Default fallback height
    }
    pub fn get_ground_height(&self, x: f32, z: f32) -> f32 {
        let chunk_x = (x as i32).div_euclid(16);
        let chunk_z = (z as i32).div_euclid(16);
        
        if let Some(chunk) = self.chunks.get(&(chunk_x, 0, chunk_z)) {
            let local_x = (x as i32).rem_euclid(16) as usize;
            let local_z = (z as i32).rem_euclid(16) as usize;
            
            for y in (0..256).rev() {
                let block_type = chunk.blocks[local_x][y][local_z];
                if block_type.is_solid() {
                    return y as f32 + 1.0;
                }
            }
        }
        
        0.0 // Default ground height if no chunk found
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
    pub fn get_chunk_neighbors(&self, cx: i32, cy: i32, cz: i32) -> Vec<(i32, i32, i32)> {
        let mut neighbors = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let target_y = cy + dy;
                    if target_y >= 0 && target_y < (WORLD_HEIGHT / 16) {
                        neighbors.push((cx + dx, target_y, cz + dz));
                    }
                }
            }
        }
        neighbors
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
            // Set block to Air first
            self.set_block_world(pos, BlockType::Air);
            
            // Get all affected chunks (3x3x3 area around the broken block)
            let mut affected = self.get_affected_chunks(pos);
            
            // Mark all affected chunks as dirty IMMEDIATELY
            for &(cx, cy, cz) in &affected {
                if let Some(chunk) = self.chunks.get_mut(&(cx, cy, cz)) {
                    chunk.mesh_dirty = true;
                    self.dirty_chunks.insert((cx, cy, cz)); // PRIORITY UPDATE
                }
            }
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
            
            // Trigger water updates if needed
            let water_affected = self.trigger_water_update(pos);
            affected.extend(water_affected);
            
            affected.sort_unstable();
            affected.dedup();
            affected
        } else {
            Vec::new()
        }
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
use winit::keyboard::KeyCode;
use glam::{Mat4, Vec4Swizzles};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemStack { 
    pub item: BlockType, 
    pub count: u8,
    pub durability: u16,
}
impl ItemStack { 
    pub fn new(item: BlockType, count: u8) -> Self { 
        Self { item, count, durability: item.get_max_durability() } 
    } 
}

pub const INVENTORY_SIZE: usize = 36; 
pub const HOTBAR_SIZE: usize = 9;

pub struct Inventory {
    pub slots: [Option<ItemStack>; INVENTORY_SIZE],
    pub selected_hotbar_slot: usize,
    pub cursor_item: Option<ItemStack>, 
    pub crafting_grid: Vec<Option<ItemStack>>, 
    pub crafting_output: Option<ItemStack>,
}
#[allow(dead_code)]
impl Inventory {
    pub fn new() -> Self { Inventory { slots: [None; INVENTORY_SIZE], selected_hotbar_slot: 0, cursor_item: None, crafting_grid: vec![None; 9], crafting_output: None } }
    pub fn get_selected_item(&self) -> Option<BlockType> { self.slots[self.selected_hotbar_slot].map(|stack| stack.item) }
    pub fn remove_one_from_hand(&mut self) { if let Some(stack) = &mut self.slots[self.selected_hotbar_slot] { if stack.count > 1 { stack.count -= 1; } else { self.slots[self.selected_hotbar_slot] = None; } } }
    
    pub fn drop_item(&mut self, drop_all: bool) -> Option<ItemStack> {
        if let Some(stack) = &mut self.slots[self.selected_hotbar_slot] {
            if drop_all {
                let ret = *stack; self.slots[self.selected_hotbar_slot] = None; return Some(ret);
            } else {
                let mut ret = *stack; ret.count = 1;
                if stack.count > 1 { stack.count -= 1; } else { self.slots[self.selected_hotbar_slot] = None; }
                return Some(ret);
            }
        }
        None
    }
    pub fn select_slot(&mut self, slot: usize) { self.selected_hotbar_slot = slot.clamp(0, HOTBAR_SIZE - 1); }
pub fn add_item(&mut self, item: BlockType) -> bool {
        if item == BlockType::Air { return false; }
        for slot in &mut self.slots { 
            if let Some(stack) = slot { 
                if stack.item == item && stack.count < 64 { 
                    stack.count += 1; 
                    return true; 
                } 
            } 
        }
        for slot in &mut self.slots { 
            if slot.is_none() { 
                *slot = Some(ItemStack::new(item, 1)); 
                return true; 
            } 
        } 
        false 
    }
pub fn check_recipes(&mut self) {
        let g: Vec<u8> = self.crafting_grid.iter().map(|s| s.map(|i| i.item as u8).unwrap_or(0)).collect();
        // 3x3 Grid: 0 1 2 / 3 4 5 / 6 7 8
        
        let mut result = None;

// 1. OAK LOG -> 4 OAK PLANKS
        if g.iter().filter(|&&id| id == 4).count() == 1 && g.iter().filter(|&&id| id != 0 && id != 4).count() == 0 {
            result = Some((BlockType::Planks, 4));
        }
        // 2. SPRUCE LOG -> 4 SPRUCE PLANKS
        if g.iter().filter(|&&id| id == 72).count() == 1 && g.iter().filter(|&&id| id != 0 && id != 72).count() == 0 {
            result = Some((BlockType::SprucePlanks, 4));
        }
        // 3. BIRCH LOG -> 4 BIRCH PLANKS
        if g.iter().filter(|&&id| id == 74).count() == 1 && g.iter().filter(|&&id| id != 0 && id != 74).count() == 0 {
            result = Some((BlockType::BirchPlanks, 4));
        }
        // 2. 2x2 PLANKS -> CRAFTING TABLE
        if g[0] == 14 && g[1] == 14 && g[3] == 14 && g[4] == 14 && g[2]==0 && g[5]==0 && g[6]==0 && g[7]==0 && g[8]==0 {
            result = Some((BlockType::CraftingTable, 1));
        }

        // 3. STICKS (2 Planks Vertical)
        let is_stick_shape = |top, bot| top == 14 && bot == 14;
        let total_items = g.iter().filter(|&&id| id != 0).count();
        if total_items == 2 {
            if is_stick_shape(g[0], g[3]) || is_stick_shape(g[1], g[4]) || is_stick_shape(g[3], g[6]) || is_stick_shape(g[4], g[7]) || is_stick_shape(g[2], g[5]) || is_stick_shape(g[5], g[8]) {
                result = Some((BlockType::Stick, 4));
            }
        }

        // 4. Complex Recipes
        if result.is_none() {
            result = match (g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7], g[8]) {
                // Tools
                (14,14,14, 0,15,0, 0,15,0) => Some((BlockType::WoodPickaxe, 1)),
                (14,14,0, 14,15,0, 0,15,0) => Some((BlockType::WoodAxe, 1)),
                (0,14,0, 0,15,0, 0,15,0) => Some((BlockType::WoodShovel, 1)),
                (0,14,0, 0,14,0, 0,15,0) => Some((BlockType::WoodSword, 1)),
                (16,16,16, 0,15,0, 0,15,0) => Some((BlockType::StonePickaxe, 1)),
                (16,16,0, 16,15,0, 0,15,0) => Some((BlockType::StoneAxe, 1)),
                (0,16,0, 0,15,0, 0,15,0) => Some((BlockType::StoneShovel, 1)),
                (0,16,0, 0,16,0, 0,15,0) => Some((BlockType::StoneSword, 1)),
                (17,17,17, 0,15,0, 0,15,0) => Some((BlockType::IronPickaxe, 1)),
                (19,19,19, 0,15,0, 0,15,0) => Some((BlockType::DiamondPickaxe, 1)),
                
                // Functional Blocks
                (0,10,0, 0,15,0, 0,0,0) => Some((BlockType::Torch, 4)),
                (16,16,16, 16,0,16, 16,16,16) => Some((BlockType::Furnace, 1)),
                (14,14,14, 14,0,14, 14,14,14) => Some((BlockType::Chest, 1)),
                (14,14,0, 14,14,0, 14,14,0) => Some((BlockType::Stick, 1)), 
                (14,14,14, 14,14,14, 0,0,0) => Some((BlockType::Stick, 2)), 
                
                // Environment
                (0,5,0, 5,5,5, 0,5,0) => Some((BlockType::OakSapling, 1)), 
                (7,0,7, 0,7,0, 7,0,7) => Some((BlockType::TNT, 1)),
                (14,14,14, 15,15,15, 14,14,14) => Some((BlockType::Bookshelf, 1)),
                _ => None
            };
        }
        
        // Shapeless Recipes (Buttons, Levers)
        if result.is_none() {
             let stone_cnt = g.iter().filter(|&&i| i == 3).count();
             let plank_cnt = g.iter().filter(|&&i| i == 14).count();
             if stone_cnt == 1 && plank_cnt == 0 && g.iter().filter(|&&i| i!=0 && i!=3).count() == 0 {
                 result = Some((BlockType::Stone, 1)); // Button
             }
        }

        self.crafting_output = result.map(|(i, c)| ItemStack::new(i, c));
    }
    
    pub fn craft(&mut self) {
        if self.crafting_output.is_some() { 
            for i in 0..9 { 
                if let Some(stack) = &mut self.crafting_grid[i] { 
                    if stack.count > 1 { stack.count -= 1; } else { self.crafting_grid[i] = None; } 
                } 
            } 
        }
    }
}

pub struct Player {
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
    pub height: f32,
    pub radius: f32,
    pub on_ground: bool,
    pub inventory: Inventory,
    pub input: PlayerInput, // ROOT FIX: Renamed 'keys' to 'input' for semantic parity
    pub hotbar: crate::Hotbar,

    // DIABOLICAL INTERPOLATION: Store previous state to kill visual jitter
    pub prev_position: Vec3,
    pub prev_rotation: Vec3,

    pub is_flying: bool,
    pub is_noclip: bool,
    pub admin_speed: f32, // NEW
    pub is_sprinting: bool,
    pub health: f32,
    pub max_health: f32,
    pub air: f32,
    pub max_air: f32,
    pub invincible_timer: f32,
    pub speed: f32,
    pub walk_time: f32,
    pub sensitivity: f32,
    pub inventory_open: bool,
pub crafting_open: bool,
pub is_dead: bool,
pub bob_timer: f32,
    pub spawn_timer: f32,
    pub cave_sound_timer: f32,
    pub grounded_latch: f32,   // Coyote Time / Hysteresis buffer
    pub jump_buffer_timer: f32, // Allows pressing jump slightly before hitting ground
    pub last_step_variant: usize,
    pub stasis: bool,
}

#[derive(Default)]
pub struct PlayerInput { 
    pub forward: bool, pub backward: bool, pub left: bool, pub right: bool, 
    pub jump: bool, // Renamed from 'up' to match main.rs usage
    pub sneak: bool, // Renamed from 'down' to match main.rs usage
    pub sprint: bool, // Added field
    pub jump_queued: bool, // DIABOLICAL FIX: Buffer the jump request to sync with physics sub-steps
}

impl PlayerInput {
    pub fn reset(&mut self) {
        self.forward = false; self.backward = false; self.left = false;
        self.right = false; self.jump = false; self.sneak = false;
        self.sprint = false;
    }
}

#[allow(dead_code)]
impl Player {
pub fn new() -> Self {
Player {
            position: Vec3::new(0.0, 100.0, 0.0),
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            height: 1.8,
            radius: 0.3,
            on_ground: false,
        inventory: Inventory::new(),
        input: PlayerInput::default(),
        hotbar: crate::Hotbar::new(),
            prev_position: Vec3::new(0.0, 100.0, 0.0),
            prev_rotation: Vec3::ZERO,
            is_flying: false,
            is_noclip: false,
            admin_speed: 1.0,
            is_sprinting: false,
            health: 10.0, // Always start with 10 hearts
            max_health: 20.0,
            air: 10.0,
            max_air: 10.0,
            invincible_timer: 0.0,
            speed: 5.0,
            walk_time: 0.0,
            sensitivity: 0.005,
            inventory_open: false,
crafting_open: false,
is_dead: false,
bob_timer: 0.0,
            spawn_timer: 0.0,
            cave_sound_timer: 15.0,
            grounded_latch: 0.0,
            jump_buffer_timer: 0.0,
            last_step_variant: 0,
            stasis: false,
        }
    }
    pub fn respawn(&mut self) { self.position = Vec3::new(0.0, 80.0, 0.0); self.velocity = Vec3::ZERO; self.health = 10.0; self.is_dead = false; self.invincible_timer = 3.0; }
    
    pub fn take_damage(&mut self, amount: f32, _damage_type: &str) {
        if self.invincible_timer > 0.0 { return; }
        self.health -= amount;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_dead = true;
        }
    }
    
        pub fn handle_input(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.input.forward = pressed, KeyCode::KeyS => self.input.backward = pressed,
            KeyCode::KeyA => self.input.left = pressed, KeyCode::KeyD => self.input.right = pressed,
            KeyCode::Space => { self.input.jump = pressed; if pressed { self.input.jump_queued = true; } },
            KeyCode::ShiftLeft => self.input.sneak = pressed,
            KeyCode::ControlLeft => self.input.sprint = pressed,
            KeyCode::Digit1 => self.inventory.select_slot(0), KeyCode::Digit2 => self.inventory.select_slot(1),
            KeyCode::Digit3 => self.inventory.select_slot(2), KeyCode::Digit4 => self.inventory.select_slot(3),
            KeyCode::Digit5 => self.inventory.select_slot(4), KeyCode::Digit6 => self.inventory.select_slot(5),
            KeyCode::Digit7 => self.inventory.select_slot(6), KeyCode::Digit8 => self.inventory.select_slot(7),
KeyCode::Digit9 => self.inventory.select_slot(8),
            _ => {}
        }
    }
    
    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        if self.is_dead || self.inventory_open { return; }
        // DIABOLICAL NOISE FILTER: Ignore hardware-level mouse shivering
        if dx.abs() < 0.0001 && dy.abs() < 0.0001 { return; }
        
        self.rotation.y += dx as f32 * self.sensitivity; 
        self.rotation.x -= dy as f32 * self.sensitivity;
        self.rotation.x = self.rotation.x.clamp(-1.55, 1.55); // Clamp pitch
    }

    pub fn capture_state(&mut self) {
        self.prev_position = self.position;
        self.prev_rotation = self.rotation;
    }
    
pub fn update(&mut self, world: &crate::engine::World, dt: f32, audio: &crate::AudioSystem, in_cave: bool) {
        if self.is_dead || self.inventory_open { return; }
        
        // OPTIMIZED PHYSICS SUB-STEPPING: 4 steps for better performance
        let substeps = 4; // Reduced from 8 for better FPS
        let sub_dt = dt.min(0.1) / substeps as f32;
        
        for _ in 0..substeps {
            // ROOT FIX: We no longer pass stale 'was_on_ground' state. 
            // The internal update now manages state transitions diabolically well.
            self.internal_update(world, sub_dt, audio, in_cave);
        }
    }

    fn internal_update(&mut self, world: &crate::engine::World, dt: f32, audio: &crate::AudioSystem, in_cave: bool) {
        if self.invincible_timer > 0.0 { self.invincible_timer -= dt; }
        
                // --- DIABOLICAL GROUNDING HYSTERESIS ---
        if self.grounded_latch > 0.0 { self.grounded_latch -= dt; }
        if self.jump_buffer_timer > 0.0 { self.jump_buffer_timer -= dt; }
        
        if self.input.jump_queued {
            self.jump_buffer_timer = 0.15;
            self.input.jump_queued = false;
        }

        // RADICAL FIX: Recalculate on_ground status IMMEDIATELY to prevent frame-lag jitter
        let ground_check = self.check_ground(world, self.position);
        self.on_ground = ground_check.is_some() || self.grounded_latch > 0.0;

        // --- CAVE AMBIENCE ---
        if in_cave {
            self.cave_sound_timer -= dt;
            if self.cave_sound_timer <= 0.0 {
                audio.play("spooky", true);
                // Use walk_time as a pseudo-random seed to vary next timing
                let mut rng = crate::engine::SimpleRng::new(self.walk_time as u64 + 1);
                self.cave_sound_timer = 45.0 + rng.next_f32() * 120.0; // Play every 45-165 seconds
            }
        } else {
            self.cave_sound_timer = 15.0; // Grace period when entering
        }

        // --- SURVIVAL MECHANICS ---
let feet_bp = BlockPos { x: self.position.x.floor() as i32, y: self.position.y.floor() as i32, z: self.position.z.floor() as i32 };
        let eye_bp = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + self.height * 0.4).floor() as i32, z: self.position.z.floor() as i32 };
        let head_bp = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + self.height * 0.9).floor() as i32, z: self.position.z.floor() as i32 };
        
// 1. DROWNING (Approx 10 seconds total air)
        if world.get_block(eye_bp).is_water() {
            self.air -= dt; // 1 unit per second
            if self.air <= 0.0 {
                self.air = 0.0;
                if self.invincible_timer <= 0.0 { self.health -= 2.0; self.invincible_timer = 1.0; }
            }
        } else {
            self.air = (self.air + dt * 2.5).min(self.max_air); // Regenerate air
        }

        // 2. LAVA DAMAGE
        if world.get_block(feet_bp) == BlockType::Lava || world.get_block(head_bp) == BlockType::Lava {
            if self.invincible_timer <= 0.0 { self.health -= 4.0; self.invincible_timer = 0.5; }
            self.velocity.y *= 0.5; // Viscosity
        }

        // 3. CACTUS DAMAGE
        let neighbors = [BlockPos{x:feet_bp.x+1, y:feet_bp.y, z:feet_bp.z}, BlockPos{x:feet_bp.x-1, y:feet_bp.y, z:feet_bp.z}, BlockPos{x:feet_bp.x, y:feet_bp.y, z:feet_bp.z+1}, BlockPos{x:feet_bp.x, y:feet_bp.y, z:feet_bp.z-1}];
        for n in neighbors { if world.get_block(n) == BlockType::Cactus {
             if (self.position.x - n.x as f32 - 0.5).abs() < 0.8 && (self.position.z - n.z as f32 - 0.5).abs() < 0.8 {
                 if self.invincible_timer <= 0.0 { self.health -= 1.0; self.invincible_timer = 0.5; }
             }
        }}

               let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize(); let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let mut move_delta = Vec3::ZERO;
        if self.input.forward { move_delta += forward; } if self.input.backward { move_delta -= forward; }
        if self.input.right { move_delta += right; } if self.input.left { move_delta -= right; }
if move_delta.length_squared() > 0.0 { 
            let mut speed_mult = if self.is_flying { self.admin_speed * 4.0 } else { 1.0 };
            if self.input.sprint && !self.is_flying { speed_mult *= 1.5; }
            move_delta = move_delta.normalize() * self.speed * speed_mult * dt; 
        }
        
        // Physics & Block Modifiers
let chest_bp = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + self.height * 0.3).floor() as i32, z: self.position.z.floor() as i32 };
        let eye_bp = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + self.height * 0.4).floor() as i32, z: self.position.z.floor() as i32 };
        let in_water = world.get_block(chest_bp).is_water() || world.get_block(eye_bp).is_water();
        let current_block = world.get_block(head_bp);
        let in_leaves = matches!(current_block, BlockType::Leaves);

if in_water {
            move_delta *= 0.65;
            if self.input.jump { 
                // Diabolical Fix: If we are near the surface, give a massive boost to "breach" onto land
                let surface_check = world.get_block(BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + 0.8).floor() as i32, z: self.position.z.floor() as i32 });
                if surface_check == BlockType::Air {
                    self.velocity.y = 9.0; // Same as a regular jump to clear 1 block
                } else {
                    self.velocity.y = (self.velocity.y + 20.0 * dt).min(4.5); 
                }
            } else if self.input.sneak {
                self.velocity.y = (self.velocity.y - 14.0 * dt).max(-4.0); 
            } else {
                self.velocity.y = (self.velocity.y - 1.5 * dt).max(-1.2); // Slower sink
            }
            self.on_ground = false;
                } else if in_leaves {
            move_delta *= 0.75; 
            self.velocity.y = (self.velocity.y - 5.0 * dt).max(-1.5); 
            if self.input.jump { self.velocity.y = 3.0; } 
            self.on_ground = false;
        } else {
            // DIABOLICAL JITTER KILLER: Only apply gravity if not grounded or jumping
            if !self.on_ground || self.velocity.y > 0.0 {
                self.velocity.y -= 28.0 * dt; 
            } else {
                self.velocity.y = -0.1; // Sticky floor force
            }

                    if self.on_ground && (self.input.forward || self.input.backward || self.input.left || self.input.right) { 
            self.bob_timer += dt;
                if self.bob_timer > 0.35 {
                    // DIABOLICAL MATERIAL DETECTION
                    let feet_pos = BlockPos { 
                        x: self.position.x.floor() as i32, 
                        y: (self.position.y - self.height * 0.5 - 0.1).floor() as i32, 
                        z: self.position.z.floor() as i32 
                    };
                    let block_below = world.get_block(feet_pos);
                    let category = block_below.get_step_sound_category();
                    
                    // Increment variant to ensure the NEXT step is different
                    self.last_step_variant = (self.last_step_variant + 1) % 5;
                    audio.play_step(category, self.last_step_variant, in_cave);
                    
                    self.bob_timer = 0.0;
                }
            }
        }

if move_delta.length_squared() > 0.0 {
             let next_x = self.position.x + move_delta.x;
             let next_z = self.position.z + move_delta.z;

             if self.is_noclip {
                 self.position.x = next_x;
                 self.position.z = next_z;
             } else {
                 if !self.check_collision_horizontal(world, Vec3::new(next_x, self.position.y, self.position.z)) { self.position.x = next_x; }
                 if !self.check_collision_horizontal(world, Vec3::new(self.position.x, self.position.y, next_z)) { self.position.z = next_z; }
             }
             self.walk_time += dt * 10.0;
        }
        
let next_y = self.position.y + self.velocity.y * dt;
        
        // DIABOLICAL JUMP LOGIC: Can we jump?
        let can_jump = (self.on_ground || self.grounded_latch > 0.0) && !self.is_flying;
        if can_jump && self.jump_buffer_timer > 0.0 {
            self.velocity.y = 9.2; // Optimized for 1.25 block vertical reach
            self.on_ground = false;
            self.grounded_latch = 0.0;
            self.jump_buffer_timer = 0.0;
            // Immediate Y update to clear the ground check zone
            self.position.y += self.velocity.y * dt;
        } else if self.velocity.y <= 0.001 {
            if let Some(ground_y) = self.check_ground(world, Vec3::new(self.position.x, next_y, self.position.z)) {
                if !self.on_ground {
                    let eye_p = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + self.height * 0.4).floor() as i32, z: self.position.z.floor() as i32 };
                    let is_submerged = world.get_block(eye_p).is_water();
                    audio.play("land", is_submerged || in_cave);
                    self.bob_timer = 0.0;
                }
                
                self.position.y = ground_y; // Pure Snap
                if !in_water && self.velocity.y < -18.0 && self.invincible_timer <= 0.0 { 
                    self.health -= (self.velocity.y.abs() - 16.0) * 0.5; 
                }
                
                self.velocity.y = 0.0; 
                // Friction lock: Stop micro-drifting
                if move_delta.length_squared() < 0.00001 { self.velocity.x = 0.0; self.velocity.z = 0.0; }
                self.on_ground = true;
                self.grounded_latch = 0.05; // RADICAL PHYSICS FIX: Reduced from 0.25 to 0.05 to kill "Walk on Air" bug
            } else { 
                self.position.y = next_y; 
                self.on_ground = false; 
            }
        } else {
            if let Some(ceil_y) = self.check_ceiling(world, Vec3::new(self.position.x, next_y, self.position.z)) {
                // FIX: No teleport. Just stop upward velocity and keep Y position below the ceiling.
                self.position.y = (ceil_y - (self.height * 0.5) - 0.01).min(self.position.y);
                self.velocity.y = 0.0;
            } else { self.position.y = next_y; }
            self.on_ground = false;
        }
if self.health <= 0.0 { self.health = 0.0; self.is_dead = true; }
    }

fn check_ground(&self, world: &World, pos: Vec3) -> Option<f32> {
        let feet_y = pos.y - self.height / 2.0;
        // DIABOLICAL RADIUS: Use a slightly larger check area (0.95) to ensure you can jump 
        // while standing on the absolute corner of a block.
        let r = self.radius * 0.95; 
        let check_points = [
            (pos.x - r, feet_y, pos.z - r), (pos.x + r, feet_y, pos.z + r), 
            (pos.x + r, feet_y, pos.z - r), (pos.x - r, feet_y, pos.z + r),
            (pos.x, feet_y, pos.z),
            // Middle-edge points for perfect corner coverage
            (pos.x - r, feet_y, pos.z), (pos.x + r, feet_y, pos.z), (pos.x, feet_y, pos.z - r), (pos.x, feet_y, pos.z + r)
        ];

        for (x, y, z) in check_points {
            let bp = BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 };
            if world.get_block(bp).is_solid() { 
                let top = bp.y as f32 + 1.0; 
                // STABLE SNAP: Increased window and bias to ensure the player sticks to blocks like glue.
                if top >= feet_y - 0.15 && top - feet_y <= 0.2 { 
                    return Some(top + self.height / 2.0); 
                } 
            }
        }
        None
    }

    fn check_ceiling(&self, world: &World, pos: Vec3) -> Option<f32> {
        let head_y = pos.y + self.height / 2.0;
        let check_points = [(pos.x, head_y, pos.z)];
        for (x, y, z) in check_points {
            let bp = BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 };
            if world.get_block(bp).is_solid() { return Some(bp.y as f32); }
        }
        None
    }

fn check_collision_horizontal(&self, world: &World, pos: Vec3) -> bool {
        let r = self.radius - 0.05; // Slightly smaller hitbox for smoother movement
        let heights = [pos.y - 0.8, pos.y, pos.y + 0.8];
        let corners = [(-r, -r), (r, r), (r, -r), (-r, r)];
        for &h in &heights {
            for &(dx, dz) in &corners {
                let bp = BlockPos { x: (pos.x + dx).floor() as i32, y: h.floor() as i32, z: (pos.z + dz).floor() as i32 };
                if world.get_block(bp).is_solid() { return true; }
            }
        }
        false
    }

    fn check_collision(&self, world: &World, pos: Vec3) -> bool {
        let min_x = (pos.x - self.radius).floor() as i32;
        let max_x = (pos.x + self.radius).floor() as i32;
        let min_y = (pos.y - self.height * 0.9).floor() as i32;
        let max_y = (pos.y + self.height * 0.1).floor() as i32;
        let min_z = (pos.z - self.radius).floor() as i32;
        let max_z = (pos.z + self.radius).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.get_block(BlockPos { x, y, z }).is_solid() {
                        return true;
                    }
                }
            }
        }
        false
    }
    
pub fn build_view_projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        let (pitch_sin, pitch_cos) = self.rotation.x.sin_cos(); 
        let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let mut eye_pos = self.position + Vec3::new(0.0, self.height * 0.4, 0.0);
        
           if self.on_ground && (self.input.forward || self.input.backward || self.input.left || self.input.right) { 
        eye_pos.y += (self.walk_time * 2.0).sin() * 0.02; 
    }
        
        let forward = Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let view = Mat4::look_at_rh(eye_pos, eye_pos + forward, Vec3::Y);
        // Tighter Far Plane to reduce geometry pressure
        let proj = Mat4::perspective_rh(75.0f32.to_radians(), aspect, 0.1, 512.0);
        
        // Correcting for WGPU coordinate system (Y-down NDC)
        let correction = Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        ]);
        
(correction * proj * view).to_cols_array_2d()
    }

    pub fn get_frustum_planes(&self, aspect: f32) -> [[f32; 4]; 6] {
        let m = glam::Mat4::from_cols_array_2d(&self.build_view_projection_matrix(aspect));
        let mut planes = [[0.0f32; 4]; 6];
        // Extract planes and NORMALIZE them for accurate distance checks
        let row4 = m.row(3);
        let rows = [m.row(0), m.row(1), m.row(2)];

        let p_raw = [
            row4 + rows[0], row4 - rows[0], // Left, Right
            row4 + rows[1], row4 - rows[1], // Bottom, Top
            row4 + rows[2], row4 - rows[2], // Near, Far
        ];

        for i in 0..6 {
            let p = p_raw[i];
            let length = p.xyz().length();
            planes[i] = (p / length).to_array();
        }
        planes
    }
}
