use winit::keyboard::KeyCode;
use glam::{Vec3, Mat4, Vec4Swizzles};
use crate::world::{World, BlockPos, BlockType};
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
    pub keys: PlayerKeys,
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
pub struct PlayerKeys { 
    pub forward: bool, pub backward: bool, pub left: bool, pub right: bool, pub up: bool, pub down: bool,
    pub jump_queued: bool, // DIABOLICAL FIX: Buffer the jump request to sync with physics sub-steps
}

impl PlayerKeys {
    pub fn reset(&mut self) {
        self.forward = false; self.backward = false; self.left = false;
        self.right = false; self.up = false; self.down = false;
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
            keys: PlayerKeys::default(),
            hotbar: crate::Hotbar::new(),
            prev_position: Vec3::new(0.0, 100.0, 0.0),
            prev_rotation: Vec3::ZERO,
            is_flying: false,
            is_noclip: false,
            admin_speed: 1.0,
            is_sprinting: false,
            health: 20.0,
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
            KeyCode::KeyW => self.keys.forward = pressed, KeyCode::KeyS => self.keys.backward = pressed,
            KeyCode::KeyA => self.keys.left = pressed, KeyCode::KeyD => self.keys.right = pressed,
            KeyCode::Space => { self.keys.up = pressed; if pressed { self.keys.jump_queued = true; } },
            KeyCode::ShiftLeft => self.keys.down = pressed,
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
    
pub fn update(&mut self, world: &crate::world::World, dt: f32, audio: &crate::AudioSystem, in_cave: bool) {
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

    fn internal_update(&mut self, world: &crate::world::World, dt: f32, audio: &crate::AudioSystem, in_cave: bool) {
        if self.invincible_timer > 0.0 { self.invincible_timer -= dt; }
        
        // --- DIABOLICAL GROUNDING HYSTERESIS ---
        if self.grounded_latch > 0.0 { self.grounded_latch -= dt; }
        if self.jump_buffer_timer > 0.0 { self.jump_buffer_timer -= dt; }
        
        if self.keys.jump_queued {
            self.jump_buffer_timer = 0.15;
            self.keys.jump_queued = false;
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
                let mut rng = crate::world::SimpleRng::new(self.walk_time as u64 + 1);
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
        if self.keys.forward { move_delta += forward; } if self.keys.backward { move_delta -= forward; }
        if self.keys.right { move_delta += right; } if self.keys.left { move_delta -= right; }
if move_delta.length_squared() > 0.0 { 
            let mut speed_mult = if self.is_flying { self.admin_speed * 4.0 } else { 1.0 };
            if self.is_sprinting && !self.is_flying { speed_mult *= 1.5; }
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
            if self.keys.up { 
                // Diabolical Fix: If we are near the surface, give a massive boost to "breach" onto land
                let surface_check = world.get_block(BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + 0.8).floor() as i32, z: self.position.z.floor() as i32 });
                if surface_check == BlockType::Air {
                    self.velocity.y = 9.0; // Same as a regular jump to clear 1 block
                } else {
                    self.velocity.y = (self.velocity.y + 20.0 * dt).min(4.5); 
                }
            } else if self.keys.down {
                self.velocity.y = (self.velocity.y - 14.0 * dt).max(-4.0); 
            } else {
                self.velocity.y = (self.velocity.y - 1.5 * dt).max(-1.2); // Slower sink
            }
            self.on_ground = false;
        } else if in_leaves {
            move_delta *= 0.75; 
            self.velocity.y = (self.velocity.y - 5.0 * dt).max(-1.5); 
            if self.keys.up { self.velocity.y = 3.0; } 
            self.on_ground = false;
        } else {
            // DIABOLICAL JITTER KILLER: Only apply gravity if not grounded or jumping
            if !self.on_ground || self.velocity.y > 0.0 {
                self.velocity.y -= 28.0 * dt; 
            } else {
                self.velocity.y = -0.1; // Sticky floor force
            }

            if self.on_ground && (move_delta.length_squared() > 0.0) {
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
        
        if self.on_ground && (self.keys.forward || self.keys.backward || self.keys.left || self.keys.right) { 
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