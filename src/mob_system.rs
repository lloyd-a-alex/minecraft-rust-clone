use serde::{Serialize, Deserialize};
use crate::world::{BlockPos, BlockType, World};
use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobType {
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Cow,
    Pig,
    Sheep,
    Chicken,
    Enderman,
    Witch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Dead,
}

#[derive(Debug, Clone)]
pub struct MobDrop {
    pub item_type: BlockType,
    pub min_count: u8,
    pub max_count: u8,
    pub chance: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub mob_type: MobType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub state: MobState,
    pub target: Option<BlockPos>,
    pub attack_cooldown: f32,
    pub wander_timer: f32,
    pub age: f32,
    pub drops: Vec<MobDrop>,
    pub experience_value: u32,
}

impl Mob {
    pub fn new(mob_type: MobType, position: Vec3) -> Self {
        let (health, max_health, drops, exp) = match mob_type {
            MobType::Zombie => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::RottenFlesh, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Carrot, min_count: 0, max_count: 1, chance: 0.1 },
                MobDrop { item_type: BlockType::Potato, min_count: 0, max_count: 1, chance: 0.1 },
            ], 5),
            MobType::Skeleton => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::Arrow, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Bone, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Bow, min_count: 0, max_count: 1, chance: 0.1 },
            ], 5),
            MobType::Creeper => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::Gunpowder, min_count: 0, max_count: 2, chance: 1.0 },
            ], 5),
            MobType::Spider => (16.0, 16.0, vec![
                MobDrop { item_type: BlockType::String, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::SpiderEye, min_count: 0, max_count: 1, chance: 0.3 },
            ], 5),
            MobType::Cow => (10.0, 10.0, vec![
                MobDrop { item_type: BlockType::Leather, min_count: 0, max_count: 3, chance: 1.0 },
                MobDrop { item_type: BlockType::Beef, min_count: 1, max_count: 3, chance: 1.0 },
            ], 3),
            MobType::Pig => (10.0, 10.0, vec![
                MobDrop { item_type: BlockType::Porkchop, min_count: 1, max_count: 3, chance: 1.0 },
            ], 3),
            MobType::Sheep => (8.0, 8.0, vec![
                MobDrop { item_type: BlockType::Wool, min_count: 1, max_count: 1, chance: 1.0 },
                MobDrop { item_type: BlockType::Mutton, min_count: 1, max_count: 2, chance: 1.0 },
            ], 3),
            MobType::Chicken => (4.0, 4.0, vec![
                MobDrop { item_type: BlockType::Feather, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Chicken, min_count: 0, max_count: 2, chance: 1.0 },
            ], 3),
            MobType::Enderman => (40.0, 40.0, vec![
                MobDrop { item_type: BlockType::EnderPearl, min_count: 0, max_count: 1, chance: 0.5 },
            ], 5),
            MobType::Witch => (26.0, 26.0, vec![
                MobDrop { item_type: BlockType::GlowstoneDust, min_count: 0, max_count: 2, chance: 0.125 },
                MobDrop { item_type: BlockType::Redstone, min_count: 0, max_count: 2, chance: 0.125 },
                MobDrop { item_type: BlockType::SpiderEye, min_count: 0, max_count: 1, chance: 0.125 },
                MobDrop { item_type: BlockType::Gunpowder, min_count: 0, max_count: 2, chance: 0.125 },
            ], 5),
        };

        Self {
            mob_type,
            position,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            health,
            max_health,
            state: MobState::Idle,
            target: None,
            attack_cooldown: 0.0,
            wander_timer: 0.0,
            age: 0.0,
            drops,
            experience_value: exp,
        }
    }

    pub fn take_damage(&mut self, amount: f32) -> bool {
        self.health -= amount;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.state = MobState::Dead;
            true
        } else {
            false
        }
    }

    pub fn get_drops(&self) -> Vec<(BlockType, u8)> {
        let mut drops = Vec::new();
        
        for drop in &self.drops {
            if rand::random::<f32>() <= drop.chance {
                let count = if drop.min_count == drop.max_count {
                    drop.min_count
                } else {
                    rand::random_range(drop.min_count..=drop.max_count)
                };
                drops.push((drop.item_type, count));
            }
        }
        
        drops
    }

    pub fn update(&mut self, world: &World, player_pos: Vec3, dt: f32) {
        self.age += dt;
        
        // Update attack cooldown
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        match self.state {
            MobState::Dead => return,
            MobState::Idle => {
                self.wander_timer -= dt;
                if self.wander_timer <= 0.0 {
                    self.state = MobState::Wandering;
                    self.wander_timer = rand::random_range(2.0..5.0);
                }
                
                // Check for player proximity
                let distance_to_player = (player_pos - self.position).length();
                let detection_range = match self.mob_type {
                    MobType::Zombie | MobType::Skeleton | MobType::Creeper => 16.0,
                    MobType::Spider => 12.0,
                    MobType::Enderman => 64.0,
                    MobType::Witch => 16.0,
                    _ => 8.0,
                };
                
                if distance_to_player < detection_range {
                    self.state = MobState::Chasing;
                    self.target = Some(BlockPos {
                        x: player_pos.x as i32,
                        y: player_pos.y as i32,
                        z: player_pos.z as i32,
                    });
                }
            }
            MobState::Wandering => {
                self.wander_timer -= dt;
                
                // Random movement
                if self.wander_timer <= 0.0 {
                    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                    self.velocity.x = angle.cos() * 0.5;
                    self.velocity.z = angle.sin() * 0.5;
                    self.wander_timer = rand::random_range(1.0..3.0);
                }
                
                // Check for player proximity
                let distance_to_player = (player_pos - self.position).length();
                let detection_range = match self.mob_type {
                    MobType::Zombie | MobType::Skeleton | MobType::Creeper => 16.0,
                    MobType::Spider => 12.0,
                    MobType::Enderman => 64.0,
                    MobType::Witch => 16.0,
                    _ => 8.0,
                };
                
                if distance_to_player < detection_range {
                    self.state = MobState::Chasing;
                    self.target = Some(BlockPos {
                        x: player_pos.x as i32,
                        y: player_pos.y as i32,
                        z: player_pos.z as i32,
                    });
                }
            }
            MobState::Chasing => {
                if let Some(target) = self.target {
                    let target_pos = Vec3::new(
                        target.x as f32 + 0.5,
                        target.y as f32,
                        target.z as f32 + 0.5,
                    );
                    
                    let direction = (target_pos - self.position).normalize();
                    let speed = match self.mob_type {
                        MobType::Zombie | MobType::Skeleton => 1.0,
                        MobType::Spider => 1.2,
                        MobType::Creeper => 0.9,
                        MobType::Enderman => 3.0,
                        MobType::Witch => 0.8,
                        MobType::Cow | MobType::Pig => 0.8,
                        MobType::Sheep => 0.8,
                        MobType::Chicken => 0.6,
                    };
                    
                    self.velocity = direction * speed;
                    
                    // Check if close enough to attack
                    let distance = (target_pos - self.position).length();
                    if distance < 2.0 && self.attack_cooldown <= 0.0 {
                        self.state = MobState::Attacking;
                        self.attack_cooldown = match self.mob_type {
                            MobType::Zombie => 1.0,
                            MobType::Skeleton => 1.5,
                            MobType::Spider => 1.0,
                            MobType::Creeper => 1.5,
                            _ => 1.0,
                        };
                    }
                } else {
                    self.state = MobState::Idle;
                }
            }
            MobState::Attacking => {
                // Attack animation and damage would be handled here
                self.state = MobState::Chasing;
            }
            MobState::Fleeing => {
                // Fleeing behavior (for passive mobs)
                let direction = (self.position - player_pos).normalize();
                let speed = 1.5;
                self.velocity = direction * speed;
                
                let distance = (player_pos - self.position).length();
                if distance > 16.0 {
                    self.state = MobState::Idle;
                }
            }
        }
        
        // Apply physics
        self.apply_physics(world, dt);
    }

    fn apply_physics(&mut self, world: &World, dt: f32) {
        // Gravity
        self.velocity.y -= 20.0 * dt;
        
        // Simple collision detection
        let next_pos = self.position + self.velocity * dt;
        
        // Ground collision
        let ground_check = BlockPos {
            x: next_pos.x.floor() as i32,
            y: (next_pos.y - 0.5).floor() as i32,
            z: next_pos.z.floor() as i32,
        };
        
        if world.get_block(ground_check).is_solid() {
            self.position.y = ground_check.y as f32 + 1.0;
            self.velocity.y = 0.0;
        } else {
            self.position.y = next_pos.y;
        }
        
        // Horizontal movement with simple collision
        let horizontal_check = BlockPos {
            x: next_pos.x.floor() as i32,
            y: (self.position.y).floor() as i32,
            z: next_pos.z.floor() as i32,
        };
        
        if !world.get_block(horizontal_check).is_solid() {
            self.position.x = next_pos.x;
            self.position.z = next_pos.z;
        }
        
        // Update rotation to face movement direction
        if self.velocity.length_squared() > 0.01 {
            self.rotation.y = self.velocity.z.atan2(self.velocity.x);
        }
    }

    pub fn get_size(&self) -> f32 {
        match self.mob_type {
            MobType::Zombie | MobType::Skeleton => 0.6,
            MobType::Creeper => 0.7,
            MobType::Spider => 0.5,
            MobType::Cow => 0.9,
            MobType::Pig => 0.8,
            MobType::Sheep => 0.8,
            MobType::Chicken => 0.4,
            MobType::Enderman => 0.6,
            MobType::Witch => 0.6,
        }
    }

    pub fn get_height(&self) -> f32 {
        match self.mob_type {
            MobType::Zombie | MobType::Skeleton => 1.8,
            MobType::Creeper => 1.7,
            MobType::Spider => 0.9,
            MobType::Cow => 1.4,
            MobType::Pig => 0.9,
            MobType::Sheep => 1.3,
            MobType::Chicken => 0.7,
            MobType::Enderman => 2.6,
            MobType::Witch => 1.8,
        }
    }
}

pub struct MobSpawner {
    pub spawn_radius: f32,
    pub max_mobs_per_chunk: usize,
    pub spawn_rate: f32, // mobs per second
    pub last_spawn: f32,
    pub mob_biomes: HashMap<MobType, Vec<String>>,
}

impl MobSpawner {
    pub fn new() -> Self {
        let mut mob_biomes = HashMap::new();
        
        // Define which biomes each mob can spawn in
        mob_biomes.insert(MobType::Zombie, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Skeleton, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Creeper, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Spider, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Cow, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Pig, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Sheep, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Chicken, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Enderman, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Witch, vec!["swamp".to_string(), "forest".to_string()]);
        
        Self {
            spawn_radius: 32.0,
            max_mobs_per_chunk: 8,
            spawn_rate: 0.1,
            last_spawn: 0.0,
            mob_biomes,
        }
    }

    pub fn try_spawn(&mut self, world: &World, player_pos: Vec3, current_time: f32) -> Option<Mob> {
        if current_time - self.last_spawn < 1.0 / self.spawn_rate {
            return None;
        }

        // Find a suitable spawn position
        let spawn_attempts = 10;
        for _ in 0..spawn_attempts {
            let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
            let distance = rand::random_range(8.0..self.spawn_radius);
            
            let spawn_x = player_pos.x + angle.cos() * distance;
            let spawn_z = player_pos.z + angle.sin() * distance;
            
            // Get ground height at this position
            let ground_y = world.get_height_at(spawn_x as i32, spawn_z as i32) as f32;
            let spawn_pos = Vec3::new(spawn_x, ground_y + 1.0, spawn_z);
            
            // Check if position is valid for spawning
            if self.is_valid_spawn_position(world, spawn_pos) {
                // Get biome at this position
                let biome = self.get_biome_at(world, spawn_x as i32, spawn_z as i32);
                
                // Choose a random mob that can spawn in this biome
                let available_mobs: Vec<MobType> = self.mob_biomes
                    .iter()
                    .filter(|(_, biomes)| biomes.contains(&biome))
                    .map(|(mob_type, _)| *mob_type)
                    .collect();
                
                if !available_mobs.is_empty() {
                    let mob_type = available_mobs[rand::random_range(0..available_mobs.len())];
                    self.last_spawn = current_time;
                    return Some(Mob::new(mob_type, spawn_pos));
                }
            }
        }
        
        None
    }

    fn is_valid_spawn_position(&self, world: &World, pos: Vec3) -> bool {
        // Check if ground is solid
        let ground_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: (pos.y - 1.0).floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if !world.get_block(ground_pos).is_solid() {
            return false;
        }
        
        // Check if spawn position is not solid
        let spawn_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: pos.y.floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if world.get_block(spawn_pos).is_solid() {
            return false;
        }
        
        // Check if above spawn position is not solid (for tall mobs)
        let above_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: (pos.y + 1.0).floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if world.get_block(above_pos).is_solid() {
            return false;
        }
        
        // Check light level (most hostile mobs spawn in darkness)
        let light_level = world.get_light_world(spawn_pos);
        if light_level > 7 {
            // Allow passive mobs to spawn in higher light
            return false; // For now, only spawn in darkness
        }
        
        true
    }

    fn get_biome_at(&self, _world: &World, _x: i32, _z: i32) -> String {
        // This would use the world's biome generation system
        // For now, return a default biome
        "plains".to_string()
    }
}
