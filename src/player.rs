use winit::keyboard::KeyCode;
use glam::{Vec3, Mat4};
use crate::world::{World, BlockPos, BlockType};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: BlockType,
    pub count: u8,
}
impl ItemStack {
    pub fn new(item: BlockType, count: u8) -> Self { Self { item, count } }
}

pub const INVENTORY_SIZE: usize = 36; 
#[allow(dead_code)]
pub const HOTBAR_SIZE: usize = 9;

#[allow(dead_code)]
pub struct Inventory {
    pub slots: [Option<ItemStack>; INVENTORY_SIZE],
    pub selected_hotbar_slot: usize,
    pub cursor_item: Option<ItemStack>, 
    pub crafting_grid: [Option<ItemStack>; 4],
    pub crafting_output: Option<ItemStack>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            slots: [None; INVENTORY_SIZE],
            selected_hotbar_slot: 0,
            cursor_item: None,
            crafting_grid: [None; 4],
            crafting_output: None,
        }
    }
    
    pub fn get_selected_item(&self) -> Option<BlockType> {
        self.slots[self.selected_hotbar_slot].map(|stack| stack.item)
    }
    
    pub fn add_item(&mut self, item: BlockType) -> bool {
        for slot in self.slots.iter_mut() {
            if let Some(stack) = slot {
                if stack.item == item && stack.count < 64 {
                    stack.count += 1;
                    return true;
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(ItemStack::new(item, 1));
                return true;
            }
        }
        false
    }

    pub fn drop_item(&mut self, drop_all: bool) -> Option<ItemStack> {
        if let Some(stack) = &mut self.slots[self.selected_hotbar_slot] {
            if drop_all || stack.count == 1 {
                let dropped = *stack;
                self.slots[self.selected_hotbar_slot] = None;
                return Some(dropped);
            } else {
                stack.count -= 1;
                return Some(ItemStack::new(stack.item, 1));
            }
        }
        None
    }
}

// FIXED: Added MouseState struct
pub struct MouseState {
    pub left: bool,
    pub right: bool,
}

pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub down: bool,
}
#[allow(dead_code)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3, // x=pitch, y=yaw
    pub keys: PlayerInput,
    pub mouse: MouseState, // FIXED: Added mouse field
    pub speed: f32,
    pub inventory: Inventory,
    pub inventory_open: bool,
    pub is_dead: bool,
    pub health: f32,
    pub height: f32,
    pub radius: f32,
    on_ground: bool,
}

impl Player {
    // FIXED: Updated new() to take position
    pub fn new(position: Vec3) -> Self {
        Player {
            position,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            keys: PlayerInput { forward: false, backward: false, left: false, right: false, jump: false, sprint: false, down: false },
            mouse: MouseState { left: false, right: false },
            speed: 4.3,
            inventory: Inventory::new(),
            inventory_open: false,
            is_dead: false,
            health: 20.0,
            height: 1.8,
            radius: 0.3,
            on_ground: false,
        }
    }

    // FIXED: Split Key and Mouse handlers
    pub fn handle_key_input(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.keys.forward = pressed,
            KeyCode::KeyS => self.keys.backward = pressed,
            KeyCode::KeyA => self.keys.left = pressed,
            KeyCode::KeyD => self.keys.right = pressed,
            KeyCode::Space => self.keys.jump = pressed,
            KeyCode::ShiftLeft => self.keys.down = pressed, // Crouch/Drop modifier
            KeyCode::ControlLeft => self.keys.sprint = pressed,
            KeyCode::KeyE if pressed => self.inventory_open = !self.inventory_open,
            KeyCode::Digit1 if pressed => self.inventory.selected_hotbar_slot = 0,
            KeyCode::Digit2 if pressed => self.inventory.selected_hotbar_slot = 1,
            KeyCode::Digit3 if pressed => self.inventory.selected_hotbar_slot = 2,
            KeyCode::Digit4 if pressed => self.inventory.selected_hotbar_slot = 3,
            KeyCode::Digit5 if pressed => self.inventory.selected_hotbar_slot = 4,
            KeyCode::Digit6 if pressed => self.inventory.selected_hotbar_slot = 5,
            KeyCode::Digit7 if pressed => self.inventory.selected_hotbar_slot = 6,
            KeyCode::Digit8 if pressed => self.inventory.selected_hotbar_slot = 7,
            KeyCode::Digit9 if pressed => self.inventory.selected_hotbar_slot = 8,
            _ => {}
        }
    }

    pub fn handle_mouse_input(&mut self, dt: f32, dx: f32, dy: f32) {
        if self.inventory_open { return; }
        let sensitivity = 0.15;
        self.rotation.y += dx * sensitivity * dt;
        self.rotation.x = (self.rotation.x - dy * sensitivity * dt).clamp(-1.5, 1.5);
    }

    pub fn update(&mut self, dt: f32, world: &World) {
        if self.is_dead { return; }
        
        let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        
        let mut move_dir = Vec3::ZERO;
        if self.keys.forward { move_dir += forward; }
        if self.keys.backward { move_dir -= forward; }
        if self.keys.right { move_dir += right; }
        if self.keys.left { move_dir -= right; }
        
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            let speed = if self.keys.sprint { self.speed * 1.5 } else { self.speed };
            self.velocity.x = move_dir.x * speed;
            self.velocity.z = move_dir.z * speed;
        } else {
            self.velocity.x *= 0.5;
            self.velocity.z *= 0.5;
        }

        if self.keys.jump && self.on_ground {
            self.velocity.y = 8.5;
            self.on_ground = false;
        }
        
        // Gravity
        self.velocity.y -= 25.0 * dt;
        
        self.handle_collisions(dt, world);
        
        // Void check
        if self.position.y < -10.0 {
            self.position = Vec3::new(0.0, 100.0, 0.0);
            self.velocity = Vec3::ZERO;
        }
    }

    fn handle_collisions(&mut self, dt: f32, world: &World) {
        // Simple AABB collision logic
        let next_pos_x = self.position + Vec3::new(self.velocity.x * dt, 0.0, 0.0);
        if !self.check_collision(next_pos_x, world) { self.position.x = next_pos_x.x; }
        else { self.velocity.x = 0.0; }

        let next_pos_z = self.position + Vec3::new(0.0, 0.0, self.velocity.z * dt);
        if !self.check_collision(next_pos_z, world) { self.position.z = next_pos_z.z; }
        else { self.velocity.z = 0.0; }

        let next_pos_y = self.position + Vec3::new(0.0, self.velocity.y * dt, 0.0);
        if !self.check_collision(next_pos_y, world) { 
            self.position.y = next_pos_y.y;
            self.on_ground = false;
        } else {
            if self.velocity.y < 0.0 { self.on_ground = true; }
            self.velocity.y = 0.0;
        }
    }

    fn check_collision(&self, pos: Vec3, world: &World) -> bool {
         let feet_y = pos.y - self.height / 2.0 + 0.1;
         let head_y = pos.y + self.height / 2.0 - 0.1;
         let check_points = [
            (pos.x-self.radius, feet_y, pos.z-self.radius),
            (pos.x+self.radius, feet_y, pos.z+self.radius),
            (pos.x+self.radius, head_y, pos.z-self.radius),
            (pos.x-self.radius, head_y, pos.z+self.radius),
        ];
        for (x, y, z) in check_points {
            let bp = BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 };
            if world.get_block(bp).is_solid() { return true; }
        }
        false
    }
    
    pub fn build_view_projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        let (pitch_sin, pitch_cos) = self.rotation.x.sin_cos();
        let (yaw_sin, yaw_cos) = self.rotation.y.sin_cos();
        let eye_pos = self.position + Vec3::new(0.0, self.height * 0.4, 0.0);
        let forward = Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let view = Mat4::look_at_rh(eye_pos, eye_pos + forward, Vec3::Y);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);
        (proj * view).to_cols_array_2d()
    }
}