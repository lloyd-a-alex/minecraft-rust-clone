use winit::keyboard::KeyCode;
use glam::{Vec3, Mat4};
use crate::world::{World, BlockPos, BlockType};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemStack { pub item: BlockType, pub count: u8 }
impl ItemStack { pub fn new(item: BlockType, count: u8) -> Self { Self { item, count } } }

pub const INVENTORY_SIZE: usize = 36; 
pub const HOTBAR_SIZE: usize = 9;

pub struct Inventory {
    pub slots: [Option<ItemStack>; INVENTORY_SIZE],
    pub selected_hotbar_slot: usize,
    pub cursor_item: Option<ItemStack>, 
    pub crafting_grid: Vec<Option<ItemStack>>, // Changed to Vec for 3x3 support
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
                let ret = *stack;
                self.slots[self.selected_hotbar_slot] = None;
                return Some(ret);
            } else {
                let mut ret = *stack;
                ret.count = 1;
                if stack.count > 1 { stack.count -= 1; } else { self.slots[self.selected_hotbar_slot] = None; }
                return Some(ret);
            }
        }
        None
    }
    pub fn select_slot(&mut self, slot: usize) { self.selected_hotbar_slot = slot.clamp(0, HOTBAR_SIZE - 1); }
    pub fn add_item(&mut self, item: BlockType) -> bool {
        for slot in &mut self.slots { if let Some(stack) = slot { if stack.item == item && stack.count < 64 { stack.count += 1; return true; } } }
        for slot in &mut self.slots { if slot.is_none() { *slot = Some(ItemStack::new(item, 1)); return true; } } false 
    }
pub fn check_recipes(&mut self) {
        // Map grid to simple ID array for matching
        let g: Vec<u8> = self.crafting_grid.iter().map(|s| s.map(|i| i.item as u8).unwrap_or(0)).collect();
        // 3x3 Grid Indices:
        // 0 1 2
        // 3 4 5
        // 6 7 8
        
        let out = match (g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7], g[8]) {
            // -- BASICS --
            (4,0,0, 0,0,0, 0,0,0) => Some((BlockType::Planks, 4)), // Wood -> 4 Planks
            (0,14,0, 0,14,0, 0,0,0) => Some((BlockType::Stick, 4)), // 2 Planks -> 4 Sticks
            (0,10,0, 0,15,0, 0,0,0) => Some((BlockType::Torch, 4)), // CoalOre + Stick -> 4 Torch (User request)
            (14,14,0, 14,14,0, 0,0,0) => Some((BlockType::CraftingTable, 1)), // 2x2 Planks -> Crafting Table

            // -- WOOD TOOLS --
            (14,14,14, 0,15,0, 0,15,0) => Some((BlockType::WoodPickaxe, 1)),
            (14,14,0, 14,15,0, 0,15,0) => Some((BlockType::WoodAxe, 1)),
            (0,14,0, 0,15,0, 0,15,0) => Some((BlockType::WoodShovel, 1)),
            (0,14,0, 0,14,0, 0,15,0) => Some((BlockType::WoodSword, 1)),

            // -- STONE TOOLS --
            (16,16,16, 0,15,0, 0,15,0) => Some((BlockType::StonePickaxe, 1)),
            (16,16,0, 16,15,0, 0,15,0) => Some((BlockType::StoneAxe, 1)),
            (0,16,0, 0,15,0, 0,15,0) => Some((BlockType::StoneShovel, 1)),
            (0,16,0, 0,16,0, 0,15,0) => Some((BlockType::StoneSword, 1)),

            // -- IRON TOOLS --
            (17,17,17, 0,15,0, 0,15,0) => Some((BlockType::IronPickaxe, 1)),
            (17,17,0, 17,15,0, 0,15,0) => Some((BlockType::IronAxe, 1)),
            (0,17,0, 0,15,0, 0,15,0) => Some((BlockType::IronShovel, 1)),
            (0,17,0, 0,17,0, 0,15,0) => Some((BlockType::IronSword, 1)),

            // -- GOLD TOOLS --
            (18,18,18, 0,15,0, 0,15,0) => Some((BlockType::GoldPickaxe, 1)),
            (18,18,0, 18,15,0, 0,15,0) => Some((BlockType::GoldAxe, 1)),
            (0,18,0, 0,15,0, 0,15,0) => Some((BlockType::GoldShovel, 1)),
            (0,18,0, 0,18,0, 0,15,0) => Some((BlockType::GoldSword, 1)),

            // -- DIAMOND TOOLS --
            (19,19,19, 0,15,0, 0,15,0) => Some((BlockType::DiamondPickaxe, 1)),
            (19,19,0, 19,15,0, 0,15,0) => Some((BlockType::DiamondAxe, 1)),
            (0,19,0, 0,15,0, 0,15,0) => Some((BlockType::DiamondShovel, 1)),
            (0,19,0, 0,19,0, 0,15,0) => Some((BlockType::DiamondSword, 1)),

            // -- INVENTORY 2x2 FALLBACK (Mapped to 3x3 slots 0,1,3,4) --
            // If user uses E-inventory, we check the top-left corner only
            (4,0,0, 0,0,0, _,_,_) => Some((BlockType::Planks, 4)), 
            (14,14,0, 14,14,0, _,_,_) => Some((BlockType::CraftingTable, 1)), 
            (0,14,0, 0,14,0, _,_,_) => Some((BlockType::Stick, 4)),

            _ => None
        };
        
        self.crafting_output = out.map(|(i, c)| ItemStack::new(i, c));
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

pub struct KeyState { pub forward: bool, pub backward: bool, pub left: bool, pub right: bool, pub up: bool, pub down: bool }

#[allow(dead_code)]
pub struct Player {
    pub position: Vec3, pub rotation: Vec3, pub velocity: Vec3,
    pub keys: KeyState, pub speed: f32, pub sensitivity: f32,
    pub inventory: Inventory, 
    pub on_ground: bool, pub walk_time: f32,
    pub height: f32, pub radius: f32,
    pub health: f32, pub max_health: f32, pub invincible_timer: f32,
pub is_dead: bool, pub right_handed: bool, pub inventory_open: bool, pub crafting_open: bool,
}
#[allow(dead_code)]
impl Player {
    pub fn new() -> Self {
        Player {
            position: Vec3::new(0.0, 80.0, 0.0), rotation: Vec3::new(0.0, -90.0f32.to_radians(), 0.0), velocity: Vec3::ZERO,
            keys: KeyState { forward: false, backward: false, left: false, right: false, up: false, down: false },
            speed: 5.0, sensitivity: 0.002, inventory: Inventory::new(),
            on_ground: false, walk_time: 0.0, height: 1.8, radius: 0.25,
            health: 20.0, max_health: 20.0, invincible_timer: 10.0, is_dead: false, right_handed: true, inventory_open: false, crafting_open: false,
        }
    }
    pub fn respawn(&mut self) { self.position = Vec3::new(0.0, 80.0, 0.0); self.velocity = Vec3::ZERO; self.health = self.max_health; self.is_dead = false; self.invincible_timer = 3.0; }
    pub fn handle_input(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.keys.forward = pressed, KeyCode::KeyS => self.keys.backward = pressed,
            KeyCode::KeyA => self.keys.left = pressed, KeyCode::KeyD => self.keys.right = pressed,
            KeyCode::Space => self.keys.up = pressed, KeyCode::ShiftLeft => self.keys.down = pressed,
            KeyCode::Digit1 => self.inventory.select_slot(0), KeyCode::Digit2 => self.inventory.select_slot(1),
            KeyCode::Digit3 => self.inventory.select_slot(2), KeyCode::Digit4 => self.inventory.select_slot(3),
            KeyCode::Digit5 => self.inventory.select_slot(4), KeyCode::Digit6 => self.inventory.select_slot(5),
            KeyCode::Digit7 => self.inventory.select_slot(6), KeyCode::Digit8 => self.inventory.select_slot(7),
            KeyCode::Digit9 => self.inventory.select_slot(8), _ => {}
        }
    }
    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        if self.is_dead || self.inventory_open { return; }
        self.rotation.y += dx as f32 * self.sensitivity; self.rotation.x -= dy as f32 * self.sensitivity;
        self.rotation.x = self.rotation.x.clamp(-1.5, 1.5);
    }
    pub fn update(&mut self, dt: f32, world: &World) {
        if self.is_dead || self.inventory_open { return; }
        let dt = dt.min(0.1); if self.invincible_timer > 0.0 { self.invincible_timer -= dt; }
        let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize(); let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let mut move_delta = Vec3::ZERO;
        if self.keys.forward { move_delta += forward; } if self.keys.backward { move_delta -= forward; }
        if self.keys.right { move_delta += right; } if self.keys.left { move_delta -= right; }
        if move_delta.length_squared() > 0.0 { move_delta = move_delta.normalize() * self.speed * dt; }
        
        // Physics
        let head_bp = BlockPos { x: self.position.x.floor() as i32, y: (self.position.y + 0.5).floor() as i32, z: self.position.z.floor() as i32 };
        let current_block = world.get_block(head_bp);
        let in_water = current_block.is_water();
        let in_leaves = matches!(current_block, BlockType::Leaves);

        if in_water {
            move_delta *= 0.7; 
            if self.keys.up { self.velocity.y = 3.0; } else if self.keys.down { self.velocity.y = -3.0; } else { self.velocity.y += (-0.5 - self.velocity.y) * (5.0 * dt).min(1.0); }
            self.on_ground = false;
        } else if in_leaves {
            move_delta *= 0.85; // Slight slowdown
            self.velocity.y *= 0.9; // Slow fall
            if self.velocity.y < -5.0 { self.velocity.y = -5.0; } // Terminal velocity in leaves
            if self.keys.up { self.velocity.y = 2.0; }
            if self.keys.up && self.on_ground { self.velocity.y = 8.0; self.on_ground = false; } // Jump out
        } else {
            self.velocity.y -= 30.0 * dt; // Gravity
            if self.on_ground && self.keys.up { self.velocity.y = 12.0; self.on_ground = false; }
        }

        if move_delta.length_squared() > 0.0 {
             let next_x = self.position.x + move_delta.x;
             if !self.check_collision_horizontal(world, Vec3::new(next_x, self.position.y, self.position.z)) { self.position.x = next_x; }
             let next_z = self.position.z + move_delta.z;
             if !self.check_collision_horizontal(world, Vec3::new(self.position.x, self.position.y, next_z)) { self.position.z = next_z; }
             self.walk_time += dt * 10.0;
        }
        
        let next_y = self.position.y + self.velocity.y * dt;
        if self.velocity.y <= 0.0 {
            if let Some(ground_y) = self.check_ground(world, Vec3::new(self.position.x, next_y, self.position.z)) {
                self.position.y = ground_y; 
                if !in_water && self.velocity.y < -14.0 && self.invincible_timer <= 0.0 { 
                    let dmg = (self.velocity.y.abs() - 12.0) * 0.5;
                    self.health -= dmg;
                    log::info!("Oof! Fall damage: {:.1} (Vel: {:.1})", dmg, self.velocity.y);
                }
                self.velocity.y = 0.0; 
                self.on_ground = true;
            } else { self.position.y = next_y; self.on_ground = false; }
        } else {
            if let Some(ceil_y) = self.check_ceiling(world, Vec3::new(self.position.x, next_y, self.position.z)) {
                self.position.y = ceil_y - self.height - 0.01; self.velocity.y = 0.0;
            } else { self.position.y = next_y; }
            self.on_ground = false;
        }
        if self.health <= 0.0 { self.health = 0.0; self.is_dead = true; }
    }

    fn check_ground(&self, world: &World, pos: Vec3) -> Option<f32> {
        let feet_y = pos.y - self.height / 2.0;
        let check_points = [(pos.x-self.radius, feet_y, pos.z-self.radius), (pos.x+self.radius, feet_y, pos.z+self.radius), (pos.x+self.radius, feet_y, pos.z-self.radius), (pos.x-self.radius, feet_y, pos.z+self.radius)];
        for (x, y, z) in check_points {
            let bp = BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 };
            if world.get_block(bp).is_solid() { let top = bp.y as f32 + 1.0; if top - feet_y <= 0.6 { return Some(top + self.height / 2.0 + 0.001); } }
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
         let feet_y = pos.y - self.height / 2.0 + 0.1; 
         let mid_y = pos.y; 
         let head_y = pos.y + self.height / 2.0 - 0.1;
         let r = self.radius;
         let corners = [(-r, -r), (r, r), (r, -r), (-r, r)];
         let heights = [feet_y, mid_y, head_y];
         for &h in &heights {
             for &(dx, dz) in &corners {
                 let bp = BlockPos { x: (pos.x + dx).floor() as i32, y: h.floor() as i32, z: (pos.z + dz).floor() as i32 };
                 if world.get_block(bp).is_solid() { return true; }
             }
         }
         false
    }
    pub fn build_view_projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        let (pitch_sin, pitch_cos) = self.rotation.x.sin_cos(); let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let mut eye_pos = self.position + Vec3::new(0.0, self.height * 0.9, 0.0);
        if self.on_ground && (self.keys.forward || self.keys.backward || self.keys.left || self.keys.right) { eye_pos.y += (self.walk_time * 2.0).sin() * 0.05; }
        let forward = Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let view = Mat4::look_at_rh(eye_pos, eye_pos + forward, Vec3::Y);
        let proj = Mat4::perspective_rh(70.0f32.to_radians(), aspect, 0.1, 1000.0);
        (proj * view).to_cols_array_2d()
    }
}