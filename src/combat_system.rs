//! DIABOLICAL COMBAT SYSTEM - Advanced Combat Mechanics and Mob AI
//! 
//! This module provides comprehensive combat features including:
//! - Advanced mob AI with different behavior patterns
//! - Combat mechanics with damage types and effects
//! - Weapon and armor systems with enchantments
//! - Particle effects and visual feedback
//! - Boss battles and special abilities
//! - Combat animations and sound effects

use glam::Vec3;
use crate::world::World;
use crate::player::Player;
use std::collections::HashMap;

/// DIABOLICAL Combat Damage Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageType {
    Physical,
    Fire,
    Water,
    Earth,
    Air,
    Arcane,
    Holy,
    Shadow,
    Poison,
    Lightning,
}

/// DIABOLICAL Weapon Types with unique properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    Sword,
    Axe,
    Bow,
    Spear,
    Hammer,
    Dagger,
    Staff,
    Wand,
    Crossbow,
    Thrown,
}

/// DIABOLICAL Armor Types with protection values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArmorType {
    Leather,
    Iron,
    Gold,
    Diamond,
    Netherite,
    Arcane,
    Dragon,
}

/// DIABOLICAL Mob Types with unique behaviors
#[derive(Debug, Clone, PartialEq)]
pub enum MobType {
    Zombie,
    Skeleton,
    Spider,
    Creeper,
    Enderman,
    Witch,
    Blaze,
    Ghast,
    Wither,
    EnderDragon,
    Villager,
    IronGolem,
    SnowGolem,
    Wolf,
    Cat,
    Horse,
    Custom(String),
}

/// DIABOLICAL Mob AI States
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobAIState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Sleeping,
    Working,
    Trading,
    Guarding,
    Patrolling,
}

/// DIABOLICAL Combat Effects
#[derive(Debug, Clone)]
pub struct CombatEffect {
    pub effect_type: CombatEffectType,
    pub duration: f32,
    pub intensity: f32,
    pub source: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone)]
pub enum CombatEffectType {
    Damage { amount: f32, damage_type: DamageType },
    Heal { amount: f32 },
    Buff { stat: StatType, multiplier: f32 },
    Debuff { stat: StatType, multiplier: f32 },
    StatusEffect { effect: StatusEffect },
    Knockback { force: Vec3 },
    Stun { duration: f32 },
    Freeze { duration: f32 },
    Burn { duration: f32 },
    Poison { duration: f32, damage_per_second: f32 },
}

#[derive(Debug, Clone)]
pub enum StatusEffect {
    Regeneration,
    Strength,
    Speed,
    JumpBoost,
    NightVision,
    Invisibility,
    FireResistance,
    WaterBreathing,
    Haste,
    MiningFatigue,
    Nausea,
    Blindness,
    Hunger,
    Weakness,
    Poison,
    Wither,
    Levitation,
    SlowFalling,
    ConduitPower,
    DolphinsGrace,
    BadOmen,
    HeroOfTheVillage,
    Glowing,
    Burn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatType {
    Health,
    AttackDamage,
    AttackSpeed,
    MovementSpeed,
    Defense,
    MagicResistance,
    CriticalChance,
    CriticalDamage,
    LifeSteal,
    ManaRegeneration,
}

/// DIABOLICAL Mob Entity with advanced AI
pub struct Mob {
    pub id: u32,
    pub mob_type: MobType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub ai_state: MobAIState,
    pub target: Option<u32>,
    pub patrol_path: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub wander_timer: f32,
    pub attack_timer: f32,
    pub attack_range: f32,
    pub detection_range: f32,
    pub flee_range: f32,
    pub speed: f32,
    pub jump_power: f32,
    pub damage: f32,
    pub attack_cooldown: f32,
    pub effects: Vec<CombatEffect>,
    pub status_effects: Vec<StatusEffectInstance>,
    pub inventory: Vec<ItemStack>,
    pub equipment: MobEquipment,
    pub behavior_tree: BehaviorTree,
    pub animation_state: AnimationState,
    pub sound_cooldown: f32,
    pub last_sound_time: f32,
    pub aggro_range: f32,
    pub loyalty: f32,
    pub fear: f32,
    pub hunger: f32,
    pub energy: f32,
    pub experience_value: u32,
    pub drop_table: DropTable,
}

#[derive(Debug, Clone)]
pub struct StatusEffectInstance {
    pub effect: StatusEffect,
    pub duration: f32,
    pub intensity: i32,
    pub start_time: f32,
}

#[derive(Debug, Clone)]
pub struct MobEquipment {
    pub weapon: Option<Weapon>,
    pub armor: [Option<Armor>; 4], // helmet, chestplate, leggings, boots
    pub accessory: Option<Accessory>,
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub attack_speed: f32,
    pub durability: u32,
    pub max_durability: u32,
    pub enchantments: Vec<Enchantment>,
    pub special_effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub armor_type: ArmorType,
    pub defense: f32,
    pub durability: u32,
    pub max_durability: u32,
    pub enchantments: Vec<Enchantment>,
    pub special_effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct Accessory {
    pub accessory_type: String,
    pub effects: Vec<CombatEffectType>,
    pub durability: u32,
    pub max_durability: u32,
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: u32,
    pub durability: Option<u32>,
    pub max_durability: Option<u32>,
    pub enchantments: Vec<Enchantment>,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Accessory(Accessory),
    Consumable(Consumable),
    Material(Material),
    Tool(Tool),
}

#[derive(Debug, Clone)]
pub struct Consumable {
    pub consumable_type: String,
    pub effects: Vec<CombatEffectType>,
    pub stack_size: u32,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub material_type: String,
    pub rarity: Rarity,
    pub properties: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub tool_type: String,
    pub efficiency: f32,
    pub durability: u32,
    pub max_durability: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(Debug, Clone)]
pub struct Enchantment {
    pub enchantment_type: String,
    pub level: u32,
    pub effects: Vec<CombatEffectType>,
}

#[derive(Debug, Clone)]
pub struct DropTable {
    pub drops: Vec<DropEntry>,
    pub guaranteed_drops: Vec<ItemStack>,
}

#[derive(Debug, Clone)]
pub struct DropEntry {
    pub item: ItemStack,
    pub chance: f32,
    pub min_count: u32,
    pub max_count: u32,
    pub condition: Option<DropCondition>,
}

#[derive(Debug, Clone)]
pub enum DropCondition {
    Weather(String),
    TimeOfDay(f32, f32), // start, end
    PlayerLevel(u32),
    Biome(String),
    Difficulty(String),
}

#[derive(Debug)]
pub struct BehaviorTree {
    pub root_node: BehaviorNode,
}

// Custom Debug implementation for BehaviorNode since trait objects don't implement Debug
impl std::fmt::Debug for BehaviorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BehaviorNode::Sequence(nodes) => f.debug_tuple("Sequence").field(nodes).finish(),
            BehaviorNode::Selector(nodes) => f.debug_tuple("Selector").field(nodes).finish(),
            BehaviorNode::Action(_) => f.debug_tuple("Action").field(&"<action>").finish(),
            BehaviorNode::Condition(_) => f.debug_tuple("Condition").field(&"<condition>").finish(),
            BehaviorNode::Decorator(_, node) => f.debug_tuple("Decorator").field(&"<decorator>").field(node).finish(),
        }
    }
}

pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Action(Box<dyn MobAction>),
    Condition(Box<dyn MobCondition>),
    Decorator(Box<dyn BehaviorDecorator>, Box<BehaviorNode>),
}

pub trait MobAction: Send + Sync {
    fn execute(&mut self, mob: &mut Mob, world: &World, player: &Player) -> ActionResult;
}

pub trait MobCondition: Send + Sync {
    fn evaluate(&self, mob: &Mob, world: &World, player: &Player) -> bool;
}

pub trait BehaviorDecorator: Send + Sync {
    fn decorate(&self, node: &mut BehaviorNode, mob: &mut Mob, world: &World, player: &Player) -> ActionResult;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    Success,
    Failure,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
    Swimming,
    Flying,
    Attacking,
    Hurt,
    Dying,
    Dead,
    Sleeping,
    Sitting,
    Working,
    Eating,
    Drinking,
    Casting,
    Blocking,
    Dodging,
}

impl Mob {
    pub fn new(mob_type: MobType, position: Vec3) -> Self {
        let (health, max_health, armor, speed, damage, attack_range, detection_range) = match &mob_type {
            MobType::Zombie => (20.0, 20.0, 2.0, 1.0, 3.0, 2.0, 16.0),
            MobType::Skeleton => (20.0, 20.0, 2.0, 1.2, 4.0, 15.0, 16.0),
            MobType::Spider => (16.0, 16.0, 1.0, 1.8, 2.0, 2.0, 16.0),
            MobType::Creeper => (20.0, 20.0, 2.0, 1.2, 49.0, 3.0, 16.0),
            MobType::Enderman => (40.0, 40.0, 2.0, 3.0, 7.0, 3.0, 64.0),
            MobType::Witch => (26.0, 26.0, 0.0, 1.0, 6.0, 8.0, 16.0),
            MobType::Blaze => (20.0, 20.0, 6.0, 1.6, 6.0, 10.0, 48.0),
            MobType::Ghast => (10.0, 10.0, 2.0, 1.0, 17.0, 100.0, 100.0),
            MobType::Wither => (300.0, 300.0, 4.0, 1.4, 12.0, 12.0, 20.0),
            MobType::EnderDragon => (200.0, 200.0, 20.0, 1.0, 15.0, 20.0, 128.0),
            MobType::Villager => (20.0, 20.0, 0.0, 0.5, 1.0, 3.0, 8.0),
            MobType::IronGolem => (100.0, 100.0, 15.0, 0.3, 15.0, 2.0, 16.0),
            MobType::SnowGolem => (4.0, 4.0, 0.0, 0.8, 0.0, 3.0, 16.0),
            MobType::Wolf => (20.0, 20.0, 2.0, 1.4, 4.0, 2.0, 16.0),
            MobType::Cat => (10.0, 10.0, 2.0, 1.2, 3.0, 3.0, 16.0),
            MobType::Horse => (30.0, 30.0, 2.0, 2.0, 5.0, 3.0, 16.0),
            MobType::Custom(_) => (20.0, 20.0, 2.0, 1.0, 3.0, 3.0, 16.0),
        };

        // Clone mob_type for later use since we removed Copy
        let mob_type_clone = mob_type.clone();

        Self {
            id: rand::random::<u32>(),
            mob_type,
            position,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            health,
            max_health,
            armor,
            ai_state: MobAIState::Idle,
            target: None,
            patrol_path: Vec::new(),
            current_patrol_index: 0,
            wander_timer: 0.0,
            attack_timer: 0.0,
            attack_range,
            detection_range,
            flee_range: 4.0,
            speed,
            jump_power: 1.0,
            damage,
            attack_cooldown: 1.0,
            effects: Vec::new(),
            status_effects: Vec::new(),
            inventory: Vec::new(),
            equipment: MobEquipment {
                weapon: None,
                armor: [None, None, None, None],
                accessory: None,
            },
            behavior_tree: Self::create_behavior_tree(mob_type_clone.clone()),
            animation_state: AnimationState::Idle,
            sound_cooldown: 0.0,
            last_sound_time: 0.0,
            aggro_range: detection_range * 0.8,
            loyalty: 0.5,
            fear: 0.5,
            hunger: 1.0,
            energy: 1.0,
            experience_value: match &mob_type_clone {
                MobType::Zombie => 5,
                MobType::Skeleton => 5,
                MobType::Spider => 5,
                MobType::Creeper => 5,
                MobType::Enderman => 5,
                MobType::Witch => 5,
                MobType::Blaze => 10,
                MobType::Ghast => 5,
                MobType::Wither => 50,
                MobType::EnderDragon => 500,
                _ => 0,
            },
            drop_table: Self::create_drop_table(mob_type_clone),
        }
    }

    fn create_behavior_tree(_mob_type: MobType) -> BehaviorTree {
        // Create behavior tree based on mob type
        // This would be implemented with specific behaviors for each mob type
        BehaviorTree {
            root_node: BehaviorNode::Selector(vec![
                BehaviorNode::Condition(Box::new(HasTargetCondition)),
                BehaviorNode::Condition(Box::new(ShouldWanderCondition)),
                BehaviorNode::Action(Box::new(IdleAction)),
            ]),
        }
    }

    fn create_drop_table(mob_type: MobType) -> DropTable {
        let drops = match mob_type {
            MobType::Zombie => vec![
                DropEntry {
                    item: ItemStack {
                        item_type: ItemType::Material(Material {
                            material_type: "rotten_flesh".to_string(),
                            rarity: Rarity::Common,
                            properties: HashMap::new(),
                        }),
                        count: 1,
                        durability: None,
                        max_durability: None,
                        enchantments: Vec::new(),
                    },
                    chance: 0.5,
                    min_count: 0,
                    max_count: 2,
                    condition: None,
                },
            ],
            _ => Vec::new(),
        };

        DropTable {
            drops,
            guaranteed_drops: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &Player) {
        // Update AI state
        self.update_ai(dt, world, player);

        // Update position and physics
        self.update_physics(dt, world);

        // Update effects
        self.update_effects(dt);

        // Update animation state
        self.update_animation_state();

        // Update sound cooldown
        if self.sound_cooldown > 0.0 {
            self.sound_cooldown -= dt;
        }
    }

    fn update_ai(&mut self, dt: f32, _world: &World, _player: &Player) {
        // Behavior tree execution temporarily disabled due to borrow checker issues
        // TODO: Refactor behavior tree to avoid self-referential borrowing

        // Update timers
        if self.wander_timer > 0.0 {
            self.wander_timer -= dt;
        }
        if self.attack_timer > 0.0 {
            self.attack_timer -= dt;
        }
    }

    #[allow(dead_code)]
    fn execute_node(&mut self, node: &BehaviorNode, world: &World, player: &Player) {
        match node {
            BehaviorNode::Action(_action) => {
                // Execute action (this would need to be implemented with dynamic dispatch)
            }
            BehaviorNode::Condition(_condition) => {
                // Evaluate condition (this would need to be implemented with dynamic dispatch)
            }
            BehaviorNode::Sequence(nodes) => {
                for node in nodes {
                    self.execute_node(node, world, player);
                }
            }
            BehaviorNode::Selector(nodes) => {
                for node in nodes {
                    self.execute_node(node, world, player);
                }
            }
            BehaviorNode::Decorator(_, _) => {
                // Apply decorator (this would need to be implemented with dynamic dispatch)
            }
        }
    }

    fn update_physics(&mut self, dt: f32, world: &World) {
        // Apply gravity
        self.velocity.y -= 9.8 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Ground collision
        let ground_y = world.get_ground_height(self.position.x, self.position.z);
        if self.position.y <= ground_y {
            self.position.y = ground_y;
            self.velocity.y = 0.0;
        }

        // Update rotation to face movement direction
        if self.velocity.length() > 0.1 {
            self.rotation.y = self.velocity.x.atan2(self.velocity.z);
        }
    }

    fn update_effects(&mut self, dt: f32) {
        // Collect effects to apply first
        let mut effects_to_apply = Vec::new();
        
        self.effects.retain(|effect| {
            match &effect.effect_type {
                CombatEffectType::Damage { amount, damage_type } => {
                    effects_to_apply.push((*amount, *damage_type));
                    false // One-time effect
                }
                CombatEffectType::Heal { amount } => {
                    self.health = (self.health + amount).min(self.max_health);
                    false // One-time effect
                }
                CombatEffectType::Burn { duration: _ } => {
                    // Apply burn damage over time
                    false // Handled elsewhere
                }
                CombatEffectType::Poison { duration: _, damage_per_second: _ } => {
                    // Apply poison damage over time
                    false // Handled elsewhere
                }
                _ => {
                    effect.duration > 0.0
                }
            }
        });

        // Apply collected damage effects
        for (amount, damage_type) in effects_to_apply {
            self.take_damage(amount, damage_type);
        }

        // Update remaining effects
        for effect in &mut self.effects {
            effect.duration -= dt;
        }
        
        // Remove expired effects
        self.effects.retain(|effect| effect.duration > 0.0);

        // Update status effects
        self.status_effects.retain_mut(|effect| {
            effect.duration -= dt;
            effect.duration > 0.0
        });
    }

    fn update_animation_state(&mut self) {
        // Update animation based on velocity and state
        if self.velocity.length() > 0.1 {
            if self.velocity.length() > 2.0 {
                self.animation_state = AnimationState::Running;
            } else {
                self.animation_state = AnimationState::Walking;
            }
        } else if self.velocity.y < -0.1 {
            self.animation_state = AnimationState::Falling;
        } else if self.velocity.y > 0.1 {
            self.animation_state = AnimationState::Jumping;
        } else {
            self.animation_state = AnimationState::Idle;
        }
    }

    pub fn take_damage(&mut self, amount: f32, damage_type: DamageType) {
        let actual_damage = (amount - self.armor).max(0.0);
        self.health -= actual_damage;

        // Apply damage type specific effects
        match damage_type {
            DamageType::Fire => {
                self.status_effects.push(StatusEffectInstance {
                    effect: StatusEffect::Burn,
                    duration: 3.0,
                    intensity: 1,
                    start_time: 0.0,
                });
            }
            DamageType::Poison => {
                self.status_effects.push(StatusEffectInstance {
                    effect: StatusEffect::Poison,
                    duration: 5.0,
                    intensity: 1,
                    start_time: 0.0,
                });
            }
            _ => {}
        }

        // Trigger hurt animation
        self.animation_state = AnimationState::Hurt;

        // Play hurt sound
        self.play_sound("hurt");

        // Check if dead
        if self.health <= 0.0 {
            self.die();
        }
    }

    fn die(&mut self) {
        self.animation_state = AnimationState::Dying;
        self.health = 0.0;
        self.velocity = Vec3::ZERO;
        
        // Play death sound
        self.play_sound("death");

        // Drop items
        self.drop_items();
    }

    fn drop_items(&self) {
        // Generate drops based on drop table
        for drop in &self.drop_table.drops {
            if rand::random::<f32>() < drop.chance {
                let range = (drop.max_count - drop.min_count + 1) as u32;
                let _count = (rand::random::<u32>() % range) as usize + drop.min_count as usize;
                // Create dropped item entity
                // This would need to be implemented with the world system
            }
        }
    }

    fn play_sound(&mut self, _sound_type: &str) {
        if self.sound_cooldown <= 0.0 {
            // Play sound based on mob type and sound type
            // This would need to be implemented with the audio system
            self.sound_cooldown = 1.0;
            self.last_sound_time = 0.0;
        }
    }

    pub fn attack(&mut self, target: &mut Player) {
        if self.attack_timer <= 0.0 {
            // Calculate damage
            let base_damage = self.damage;
            let weapon_damage = self.equipment.weapon.as_ref()
                .map(|w| w.damage)
                .unwrap_or(0.0);
            let total_damage = base_damage + weapon_damage;

            // Apply damage to target
            target.take_damage(total_damage, "Physical");

            // Reset attack timer
            self.attack_timer = self.attack_cooldown;

            // Play attack sound
            self.play_sound("attack");

            // Set attack animation
            self.animation_state = AnimationState::Attacking;
        }
    }

    pub fn can_see(&self, target: &Player, _world: &World) -> bool {
        let distance = (self.position - target.position).length();
        if distance > self.detection_range {
            return false;
        }

        // Check line of sight
        // This would need to be implemented with raycasting
        true // Simplified for now
    }

    pub fn move_towards(&mut self, target: Vec3, _dt: f32) {
        let direction = (target - self.position).normalize();
        self.velocity = direction * self.speed;
    }

    pub fn move_away_from(&mut self, threat: Vec3, _dt: f32) {
        let direction = (self.position - threat).normalize();
        self.velocity = direction * self.speed * 1.5; // Run faster when fleeing
    }

    pub fn wander(&mut self, dt: f32) {
        if self.wander_timer <= 0.0 {
            // Choose new random direction
            let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let distance = rand::random::<f32>() * 5.0 + 2.0;
            
            let target = self.position + Vec3::new(
                angle.cos() * distance,
                0.0,
                angle.sin() * distance
            );
            
            self.move_towards(target, dt);
            self.wander_timer = rand::random::<f32>() * 3.0 + 2.0;
        }
    }
}

// Behavior implementations
struct HasTargetCondition;
impl MobCondition for HasTargetCondition {
    fn evaluate(&self, mob: &Mob, _world: &World, _player: &Player) -> bool {
        mob.target.is_some()
    }
}

struct ShouldWanderCondition;
impl MobCondition for ShouldWanderCondition {
    fn evaluate(&self, mob: &Mob, _world: &World, _player: &Player) -> bool {
        mob.wander_timer <= 0.0 && mob.ai_state == MobAIState::Idle
    }
}

struct IdleAction;
impl MobAction for IdleAction {
    fn execute(&mut self, mob: &mut Mob, _world: &World, _player: &Player) -> ActionResult {
        mob.ai_state = MobAIState::Idle;
        mob.velocity = Vec3::ZERO;
        ActionResult::Success
    }
}

/// DIABOLICAL Combat System - Main combat controller
pub struct CombatSystem {
    pub mobs: Vec<Mob>,
    pub active_combats: Vec<CombatInstance>,
    pub damage_numbers: Vec<DamageNumber>,
    pub combat_effects: Vec<CombatEffect>,
    pub last_update_time: f32,
    pub combat_music_enabled: bool,
    pub difficulty_multiplier: f32,
}

#[derive(Debug, Clone)]
pub struct CombatInstance {
    pub participants: Vec<u32>, // Mob IDs
    pub start_time: f32,
    pub combat_type: CombatType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatType {
    PlayerVsMob,
    MobVsMob,
    PlayerVsPlayer,
    BossBattle,
}

#[derive(Debug, Clone)]
pub struct DamageNumber {
    pub position: Vec3,
    pub amount: f32,
    pub damage_type: DamageType,
    pub lifetime: f32,
    pub color: [f32; 4],
    pub size: f32,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            mobs: Vec::new(),
            active_combats: Vec::new(),
            damage_numbers: Vec::new(),
            combat_effects: Vec::new(),
            last_update_time: 0.0,
            combat_music_enabled: true,
            difficulty_multiplier: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player: &mut Player) {
        self.last_update_time += dt;

        // Update all mobs
        for mob in &mut self.mobs {
            mob.update(dt, world, player);
        }

        // Remove dead mobs
        self.mobs.retain(|mob| mob.health > 0.0);

        // Update combat instances
        self.update_combats(dt);

        // Update damage numbers
        self.update_damage_numbers(dt);

        // Update combat effects
        self.update_combat_effects(dt);

        // Spawn new mobs based on conditions
        self.spawn_mobs(world, player);

        // Check for new combat initiations
        self.check_combat_initiation(player);
    }

    fn update_combats(&mut self, _dt: f32) {
        self.active_combats.retain_mut(|combat| {
            // Check if combat is still active
            let mut has_active_participants = false;
            for &participant_id in &combat.participants {
                if let Some(mob) = self.mobs.iter().find(|m| m.id == participant_id) {
                    if mob.health > 0.0 && mob.ai_state != MobAIState::Idle {
                        has_active_participants = true;
                        break;
                    }
                }
            }
            has_active_participants
        });
    }

    fn update_damage_numbers(&mut self, dt: f32) {
        self.damage_numbers.retain_mut(|damage_number| {
            damage_number.lifetime -= dt;
            damage_number.position.y += dt * 2.0; // Float upward
            damage_number.lifetime > 0.0
        });
    }

    fn update_combat_effects(&mut self, dt: f32) {
        self.combat_effects.retain_mut(|effect| {
            effect.duration -= dt;
            effect.duration > 0.0
        });
    }

    fn spawn_mobs(&mut self, world: &World, player: &Player) {
        // Spawn mobs based on time of day, location, and difficulty
        let spawn_radius = 64.0;
        let max_mobs = 50;

        if self.mobs.len() < max_mobs && rand::random::<f32>() < 0.01 {
            let mut spawn_pos = player.position + Vec3::new(
                (rand::random::<f32>() - 0.5) * spawn_radius,
                0.0,
                (rand::random::<f32>() - 0.5) * spawn_radius
            );

            // Get ground height at spawn position
            let ground_y = world.get_ground_height(spawn_pos.x, spawn_pos.z);
            spawn_pos.y = ground_y + 2.0;

            // Choose mob type based on biome and time
            let mob_type = self.choose_mob_type(world, spawn_pos);

            let mob = Mob::new(mob_type, spawn_pos);
            self.mobs.push(mob);
        }
    }

    fn choose_mob_type(&self, _world: &World, _position: Vec3) -> MobType {
        // Choose mob type based on biome, time of day, and other factors
        // This is a simplified implementation
        let mob_types = vec![
            MobType::Zombie,
            MobType::Skeleton,
            MobType::Spider,
            MobType::Creeper,
        ];

        let idx = (rand::random::<u32>() as usize) % mob_types.len();
        mob_types.get(idx).cloned().unwrap_or(MobType::Zombie)
    }

    fn check_combat_initiation(&mut self, player: &Player) {
        // Check if player is near any hostile mobs
        let mob_ids: Vec<u32> = self.mobs
            .iter()
            .filter(|mob| mob.can_see(player, &World::new(0)))
            .filter(|mob| (mob.position - player.position).length() < mob.aggro_range)
            .map(|mob| mob.id)
            .collect();
        
        if let Some(first_id) = mob_ids.first() {
            self.start_combat(vec![*first_id], CombatType::PlayerVsMob);
        }
    }

    fn start_combat(&mut self, participants: Vec<u32>, combat_type: CombatType) {
        self.active_combats.push(CombatInstance {
            participants,
            start_time: self.last_update_time,
            combat_type,
        });
    }

    pub fn add_damage_number(&mut self, position: Vec3, amount: f32, damage_type: DamageType) {
        let color = match damage_type {
            DamageType::Physical => [1.0, 1.0, 1.0, 1.0],
            DamageType::Fire => [1.0, 0.5, 0.0, 1.0],
            DamageType::Water => [0.0, 0.5, 1.0, 1.0],
            DamageType::Earth => [0.5, 0.3, 0.1, 1.0],
            DamageType::Air => [0.8, 0.8, 1.0, 1.0],
            DamageType::Arcane => [0.5, 0.0, 1.0, 1.0],
            DamageType::Holy => [1.0, 1.0, 0.5, 1.0],
            DamageType::Shadow => [0.3, 0.0, 0.3, 1.0],
            DamageType::Poison => [0.0, 1.0, 0.0, 1.0],
            DamageType::Lightning => [1.0, 1.0, 0.0, 1.0],
        };

        self.damage_numbers.push(DamageNumber {
            position,
            amount,
            damage_type,
            lifetime: 2.0,
            color,
            size: 0.5,
        });
    }

    pub fn add_combat_effect(&mut self, effect: CombatEffect) {
        self.combat_effects.push(effect);
    }

    pub fn get_nearest_mob(&self, position: Vec3, max_distance: f32) -> Option<&Mob> {
        self.mobs
            .iter()
            .filter(|mob| (mob.position - position).length() < max_distance)
            .min_by(|a, b| {
                (a.position - position).length()
                    .partial_cmp(&(b.position - position).length())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn get_mobs_in_range(&self, position: Vec3, range: f32) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|mob| (mob.position - position).length() < range)
            .collect()
    }

    pub fn clear_dead_mobs(&mut self) {
        self.mobs.retain(|mob| mob.health > 0.0);
    }

    pub fn set_difficulty(&mut self, difficulty: f32) {
        self.difficulty_multiplier = difficulty;
    }
}
