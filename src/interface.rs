// DIABOLICAL UI SYSTEM - Advanced User Interface Management
// 
// This module provides comprehensive UI management with support for:
// - Dynamic menu systems with animations
// - Advanced HUD with real-time information
// - Interactive crafting interfaces
// - Particle effects and visual feedback
// - Responsive design for different screen sizes

use glam::Vec3;
use crate::engine::{BlockPos, BlockType};
use crate::engine::Player;
use crate::graphics::Renderer;
use std::collections::HashMap;

/// DIABOLICAL UI Element - A single UI component
#[derive(Debug, Clone)]
pub struct UIElement {
    pub element_type: UIElementType,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub visible: bool,
    pub enabled: bool,
}

impl UIElement {
    pub fn new(element_type: UIElementType) -> Self {
        Self {
            element_type,
            position: [0.0, 0.0],
            size: [0.1, 0.1],
            visible: true,
            enabled: true,
        }
    }
}

/// DIABOLICAL UI Element Types
#[derive(Debug, Clone)]
pub enum UIElementType {
    Button { text: String, action: UIAction },
    Label { text: String, color: [f32; 4] },
    ProgressBar { progress: f32, color: [f32; 4] },
    InventorySlot { item: Option<BlockType>, count: u32, selected: bool },
    CraftingGrid { size: (usize, usize) },
    Crosshair { style: CrosshairStyle },
}

#[derive(Debug, Clone)]
pub enum CrosshairStyle {
    Simple,
    Diabolical,
    Animated,
    Custom { texture: u32 },
}

#[derive(Debug, Clone)]
pub enum UIAction {
    Navigate(String),
    Craft(BlockType),
    PlaceBlock(BlockType),
    BreakBlock,
    OpenInventory,
    CloseInventory,
    ToggleFly,
    ToggleNoclip,
    TeleportSurface,
    SpawnMob(String),
    ChangeWeather(String),
    SetTimeOfDay(f32),
}

/// DIABOLICAL UI Container with advanced layout management
pub struct UIContainer {
    pub elements: Vec<UIElement>,
    pub layout: UILayout,
    pub visible: bool,
    pub animated: bool,
    pub animation_progress: f32,
    pub background_color: [f32; 4],
    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Debug, Clone)]
pub enum UILayout {
    Vertical { spacing: f32 },
    Horizontal { spacing: f32 },
    Grid { columns: usize, spacing: f32 },
    Absolute,
    Centered,
}

impl UIContainer {
    pub fn new(layout: UILayout) -> Self {
        Self {
            elements: Vec::new(),
            layout,
            visible: true,
            animated: false,
            animation_progress: 0.0,
            background_color: [0.1, 0.1, 0.1, 0.8],
            border_radius: 8.0,
            padding: 10.0,
        }
    }

    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn animate_in(&mut self) {
        self.animated = true;
        self.animation_progress = 0.0;
    }

    pub fn animate_out(&mut self) {
        self.animated = true;
        self.animation_progress = 1.0;
    }
}

/// DIABOLICAL Advanced HUD System
pub struct AdvancedHUD {
    pub containers: HashMap<String, UIContainer>,
    pub crosshair_style: CrosshairStyle,
    pub show_debug_info: bool,
    pub show_minimap: bool,
    pub show_compass: bool,
    pub show_weather: bool,
    pub animation_time: f32,
    pub particle_effects: Vec<ParticleEffect>,
}

#[derive(Debug, Clone)]
pub struct ParticleEffect {
    pub effect_type: ParticleEffectType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub color: [f32; 4],
    pub size: f32,
}

#[derive(Debug, Clone)]
pub enum ParticleEffectType {
    Explosion,
    Smoke,
    Fire,
    Sparkle,
    Magic,
    Blood,
    WaterSplash,
    LeafFall,
}

impl AdvancedHUD {
    pub fn new() -> Self {
        let mut containers = HashMap::new();
        
        // Main HUD container
        containers.insert("main".to_string(), UIContainer::new(UILayout::Vertical { spacing: 5.0 }));
        
        // Debug info container
        containers.insert("debug".to_string(), UIContainer::new(UILayout::Vertical { spacing: 2.0 }));
        
        // Minimap container
        containers.insert("minimap".to_string(), UIContainer::new(UILayout::Absolute));
        
        Self {
            containers,
            crosshair_style: CrosshairStyle::Diabolical,
            show_debug_info: false,
            show_minimap: false,
            show_compass: false,
            show_weather: false,
            animation_time: 0.0,
            particle_effects: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, player: &Player, world: &crate::engine::World) {
        self.animation_time += dt;
        
        // Update animations
        for container in self.containers.values_mut() {
            if container.animated {
                if container.animation_progress < 1.0 {
                    container.animation_progress = (container.animation_progress + dt * 2.0).min(1.0);
                } else {
                    container.animation_progress = 0.0;
                    container.animated = false;
                }
            }
        }

        // Update particle effects
        self.particle_effects.retain(|effect| {
            effect.lifetime > 0.0
        });
        
        for effect in &mut self.particle_effects {
            effect.lifetime -= dt;
            effect.position += effect.velocity * dt;
            effect.velocity.y -= 9.8 * dt; // Gravity
        }

        // Add ambient particle effects
        if world.get_block(BlockPos { 
            x: player.position.x as i32, 
            y: (player.position.y + 2.0) as i32, 
            z: player.position.z as i32 
        }).is_water() {
            if rand::random::<f32>() < 0.1 {
                self.add_particle_effect(ParticleEffect {
                    effect_type: ParticleEffectType::WaterSplash,
                    position: player.position + Vec3::new(
                        (rand::random::<f32>() - 0.5) * 0.5,
                        player.position.y + 1.8,
                        (rand::random::<f32>() - 0.5) * 0.5
                    ),
                    velocity: Vec3::new(
                        (rand::random::<f32>() - 0.5) * 0.2,
                        rand::random::<f32>() * 0.5,
                        (rand::random::<f32>() - 0.5) * 0.2
                    ),
                    lifetime: 2.0,
                    color: [0.3, 0.6, 1.0, 0.8],
                    size: 0.1,
                });
            }
        }
    }

    pub fn add_particle_effect(&mut self, effect: ParticleEffect) {
        self.particle_effects.push(effect);
    }

    pub fn create_explosion(&mut self, position: Vec3, intensity: f32) {
        let particle_count = (intensity * 50.0) as usize;
        for _ in 0..particle_count {
            let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let speed = rand::random::<f32>() * intensity * 5.0;
            
            self.add_particle_effect(ParticleEffect {
                effect_type: ParticleEffectType::Explosion,
                position: position,
                velocity: Vec3::new(
                    angle.cos() * speed,
                    rand::random::<f32>() * intensity * 3.0,
                    angle.sin() * speed
                ),
                lifetime: rand::random::<f32>() * 2.0 + 1.0,
                color: [1.0, 0.5, 0.0, 1.0],
                size: rand::random::<f32>() * 0.3 + 0.1,
            });
        }
    }

    pub fn render(&self, renderer: &mut Renderer, player: &Player, win_size: (u32, u32)) {
        // Render main HUD elements
        if let Some(main_container) = self.containers.get("main") {
            if main_container.visible {
                self.render_container(renderer, main_container, win_size);
            }
        }

        // Render debug info
        if self.show_debug_info {
            if let Some(debug_container) = self.containers.get("debug") {
                self.render_debug_info(renderer, debug_container, player, win_size);
            }
        }

        // Render crosshair
        self.render_crosshair(renderer, win_size);

        // Render particle effects
        self.render_particle_effects(renderer);
    }

    fn render_container(&self, renderer: &mut Renderer, container: &UIContainer, win_size: (u32, u32)) {
        let alpha = if container.animated {
            if container.animation_progress < 0.5 {
                container.animation_progress * 2.0
            } else {
                2.0 - container.animation_progress
            }
        } else {
            1.0
        };

        // Render background
        if alpha > 0.01 {
            let _bg_color = [
                container.background_color[0],
                container.background_color[1],
                container.background_color[2],
                container.background_color[3] * alpha,
            ];
            
            renderer.add_ui_quad(
                &mut Vec::new(),
                &mut Vec::new(),
                &mut 0,
                -1.0 + container.border_radius / win_size.0 as f32,
                -1.0 + container.border_radius / win_size.1 as f32,
                2.0 - (container.border_radius * 2.0) / win_size.0 as f32,
                2.0 - (container.border_radius * 2.0) / win_size.1 as f32,
                240, // Background texture
            );
        }
    }

    fn render_debug_info(&self, renderer: &mut Renderer, _container: &UIContainer, player: &Player, _win_size: (u32, u32)) {
        let debug_text = format!(
            "POS: ({:.1}, {:.1}, {:.1})\nVEL: ({:.2}, {:.2}, {:.2})\nFPS: {:.0}\nCHUNKS: {}\nENTITIES: {}",
            player.position.x, player.position.y, player.position.z,
            player.velocity.x, player.velocity.y, player.velocity.z,
            renderer.fps,
            renderer.chunk_meshes.len(),
            0 // TODO: Get actual entity count
        );

        renderer.draw_text(
            &debug_text,
            -0.95,
            0.9,
            0.02,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut 0,
        );
    }

    fn render_crosshair(&self, renderer: &mut Renderer, _win_size: (u32, u32)) {
        match self.crosshair_style {
            CrosshairStyle::Simple => {
                renderer.add_ui_quad(
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut 0,
                    -0.01, -0.01, 0.02, 0.02,
                    255, // White crosshair
                );
            }
            CrosshairStyle::Diabolical => {
                // Animated diabolical crosshair with rotation
                let rotation = (self.animation_time * 2.0).sin() * 0.1;
                let size = 0.03 + rotation.abs() * 0.01;
                
                // Main crosshair
                renderer.add_ui_quad(
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut 0,
                    -size, -0.002, size * 2.0, 0.004,
                    255,
                );
                renderer.add_ui_quad(
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut 0,
                    -0.002, -size, 0.004, size * 2.0,
                    255,
                );
                
                // Animated corners
                for i in 0..4 {
                    let angle = i as f32 * std::f32::consts::PI / 2.0 + self.animation_time * 3.0;
                    let distance = 0.05 + rotation * 0.02;
                    let x = angle.cos() * distance;
                    let y = angle.sin() * distance;
                    
                    renderer.add_ui_quad(
                        &mut Vec::new(),
                        &mut Vec::new(),
                        &mut 0,
                        x - 0.005, y - 0.005, 0.01, 0.01,
                        256, // Red corners
                    );
                }
            }
            CrosshairStyle::Animated => {
                let pulse = (self.animation_time * 4.0).sin() * 0.2 + 0.8;
                let size = 0.02 * pulse;
                
                renderer.add_ui_quad(
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut 0,
                    -size, -size, size * 2.0, size * 2.0,
                    255,
                );
            }
            CrosshairStyle::Custom { texture } => {
                renderer.add_ui_quad(
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut 0,
                    -0.02, -0.02, 0.04, 0.04,
                    texture,
                );
            }
        }
    }

    fn render_particle_effects(&self, renderer: &mut Renderer) {
        for effect in &self.particle_effects {
            let screen_pos = self.world_to_screen(effect.position, renderer);
            
            renderer.add_ui_quad(
                &mut Vec::new(),
                &mut Vec::new(),
                &mut 0,
                screen_pos.x - effect.size / 2.0,
                screen_pos.y - effect.size / 2.0,
                effect.size,
                effect.size,
                257, // Particle texture
            );
        }
    }

    fn world_to_screen(&self, world_pos: Vec3, _renderer: &Renderer) -> Vec3 {
        // Simple world-to-screen projection
        // TODO: Implement proper camera projection
        Vec3::new(
            world_pos.x * 0.1,
            world_pos.y * 0.1,
            0.0
        )
    }
}

/// DIABOLICAL Menu System with advanced animations
pub struct DiabolicalMenuSystem {
    pub active_menus: Vec<String>,
    pub menu_stack: Vec<UIContainer>,
    pub current_focus: Option<usize>,
    pub animation_time: f32,
    pub background_blur: f32,
}

impl DiabolicalMenuSystem {
    pub fn new() -> Self {
        Self {
            active_menus: Vec::new(),
            menu_stack: Vec::new(),
            current_focus: None,
            animation_time: 0.0,
            background_blur: 0.0,
        }
    }

    pub fn open_menu(&mut self, menu_name: &str, mut container: UIContainer) {
        self.active_menus.push(menu_name.to_string());
        container.animate_in();
        self.menu_stack.push(container);
        self.current_focus = Some(0);
        self.background_blur = 0.0;
    }

    pub fn close_menu(&mut self) {
        if let Some(container) = self.menu_stack.last_mut() {
            container.animate_out();
        }
        self.background_blur = 1.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_time += dt;
        
        // Update background blur
        if self.background_blur > 0.0 {
            self.background_blur = (self.background_blur - dt * 2.0).max(0.0);
        }

        // Remove closed menus
        self.menu_stack.retain(|container| {
            !(!container.visible && !container.animated)
        });
        
        // Update active menus list
        self.active_menus.clear();
        for (i, container) in self.menu_stack.iter().enumerate() {
            if container.visible || container.animated {
                self.active_menus.push(format!("menu_{}", i));
            }
        }
    }

    pub fn handle_input(&mut self, input: UIInput) -> Option<UIAction> {
        // Handle menu navigation and interactions
        match input {
            UIInput::Navigate(direction) => {
                if let Some(focus) = self.current_focus {
                    match direction {
                        NavigationDirection::Up => {
                            if focus > 0 {
                                self.current_focus = Some(focus - 1);
                            }
                        }
                        NavigationDirection::Down => {
                            if let Some(container) = self.menu_stack.last() {
                                if focus < container.elements.len() - 1 {
                                    self.current_focus = Some(focus + 1);
                                }
                            }
                        }
                        NavigationDirection::Left | NavigationDirection::Right => {
                            // Horizontal navigation not yet implemented
                        }
                    }
                }
            }
            UIInput::Select => {
                if let Some(focus) = self.current_focus {
                    if let Some(container) = self.menu_stack.last() {
                        if let Some(element) = container.elements.get(focus) {
                            return self.extract_action_from_element(element);
                        }
                    }
                }
            }
            UIInput::Back => {
                self.close_menu();
            }
            UIInput::Text(_) => {
                // Text input handling not yet implemented
            }
        }
        None
    }

    fn extract_action_from_element(&self, _element: &UIElement) -> Option<UIAction> {
        // Extract action from UI element based on its type and properties
        // This would need to be implemented based on the actual UI element structure
        None
    }
}

#[derive(Debug, Clone)]
pub enum UIInput {
    Navigate(NavigationDirection),
    Select,
    Back,
    Text(String),
}

#[derive(Debug, Clone)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub player_name: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub message_type: ChatMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageType {
    Player,
    System,
    Command,
    Death,
    Achievement,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub affected_systems: Vec<String>,
}

pub struct ChatSystem {
    pub messages: Vec<ChatMessage>,
    pub max_messages: usize,
    pub commands: HashMap<String, Box<dyn CommandHandler>>,
    pub is_chat_open: bool,
    pub input_buffer: String,
    pub cursor_position: usize,
}

impl ChatSystem {
    pub fn new() -> Self {
        let mut chat = Self {
            messages: Vec::new(),
            max_messages: 100,
            commands: HashMap::new(),
            is_chat_open: false,
            input_buffer: String::new(),
            cursor_position: 0,
        };

        // Register built-in commands
        chat.register_command("help", HelpCommand::new());
        chat.register_command("time", TimeCommand::new());
        chat.register_command("weather", WeatherCommand::new());
        chat.register_command("clear", ClearCommand::new());
        chat.register_command("seed", SeedCommand::new());
        chat.register_command("tp", TeleportCommand::new());
        chat.register_command("gamemode", GamemodeCommand::new());
        chat.register_command("give", GiveCommand::new());
        chat.register_command("spawn", SpawnCommand::new());
        chat.register_command("kill", KillCommand::new());

        chat
    }

    pub fn add_message(&mut self, player_name: &str, message: &str, message_type: ChatMessageType) {
        let chat_message = ChatMessage {
            player_name: player_name.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now(),
            message_type,
        };

        self.messages.push(chat_message);

        // Remove old messages if we exceed the limit
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn send_message(&mut self, player_name: &str, message: &str) -> Option<CommandResult> {
        if message.starts_with('/') {
            // This is a command
            let parts: Vec<&str> = message[1..].split_whitespace().collect();
            if parts.is_empty() {
                return Some(CommandResult {
                    success: false,
                    message: "No command specified. Use /help for available commands.".to_string(),
                    affected_systems: vec![],
                });
            }

            let command_name = parts[0].to_lowercase();
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            if let Some(command_handler) = self.commands.get(&command_name) {
                let result = command_handler.execute(&args);
                
                // Log command execution
                self.add_message("System", &result.message, ChatMessageType::Command);
                
                Some(result)
            } else {
                Some(CommandResult {
                    success: false,
                    message: format!("Unknown command: {}. Use /help for available commands.", command_name),
                    affected_systems: vec![],
                })
            }
        } else {
            // This is a regular chat message
            self.add_message(player_name, message, ChatMessageType::Player);
            None
        }
    }

    pub fn register_command(&mut self, name: &str, handler: Box<dyn CommandHandler>) {
        self.commands.insert(name.to_lowercase(), handler);
    }

    pub fn toggle_chat(&mut self) {
        self.is_chat_open = !self.is_chat_open;
        if self.is_chat_open {
            self.input_buffer.clear();
            self.cursor_position = 0;
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Option<CommandResult> {
        if input.is_empty() {
            return None;
        }

        let result = self.send_message("Player", input);
        self.input_buffer.clear();
        self.cursor_position = 0;
        result
    }

    pub fn get_recent_messages(&self, count: usize) -> Vec<&ChatMessage> {
        let start = if self.messages.len() > count {
            self.messages.len() - count
        } else {
            0
        };
        self.messages[start..].iter().collect()
    }

    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.add_message("System", "Chat cleared.", ChatMessageType::System);
    }
}

pub trait CommandHandler {
    fn execute(&self, args: &[String]) -> CommandResult;
    fn get_help(&self) -> String;
    fn get_usage(&self) -> String;
}

// Built-in Commands

pub struct HelpCommand;
impl HelpCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for HelpCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            CommandResult {
                success: true,
                message: "Available commands: /help, /time, /weather, /clear, /seed, /tp, /gamemode, /give, /spawn, /kill. Use /help <command> for specific help.".to_string(),
                affected_systems: vec!["chat".to_string()],
            }
        } else {
            let help_text = match args[0].as_str() {
                "time" => "/time set <day|night|dawn|dusk|noon|midnight> - Sets the time of day",
                "weather" => "/weather <clear|rain|thunder|snow> - Changes the weather",
                "clear" => "/clear - Clears the chat history",
                "seed" => "/seed - Shows the world seed",
                "tp" => "/tp <x> <y> <z> - Teleports to coordinates",
                "gamemode" => "/gamemode <0|1|2> - Sets game mode (0=survival, 1=creative, 2=adventure)",
                "give" => "/give <item> [count] - Gives items to player",
                "spawn" => "/spawn <mob> [count] - Spawns mobs",
                "kill" => "/kill [target] - Kills entities (player or all mobs)",
                _ => "Unknown command. Use /help for available commands.",
            };
            
            CommandResult {
                success: true,
                message: help_text.to_string(),
                affected_systems: vec!["chat".to_string()],
            }
        }
    }

    fn get_help(&self) -> String {
        "Shows help for commands".to_string()
    }

    fn get_usage(&self) -> String {
        "/help [command]".to_string()
    }
}

pub struct TimeCommand;
impl TimeCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for TimeCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.len() < 2 || args[0] != "set" {
            return CommandResult {
                success: false,
                message: "Usage: /time set <day|night|dawn|dusk|noon|midnight>".to_string(),
                affected_systems: vec![],
            };
        }

        let time_of_day = match args[1].as_str() {
            "day" => "morning",
            "night" => "night",
            "dawn" => "dawn",
            "dusk" => "dusk",
            "noon" => "noon",
            "midnight" => "midnight",
            _ => {
                return CommandResult {
                    success: false,
                    message: "Invalid time. Use: day, night, dawn, dusk, noon, or midnight".to_string(),
                    affected_systems: vec![],
                };
            }
        };

        CommandResult {
            success: true,
            message: format!("Time set to {}", time_of_day),
            affected_systems: vec!["time".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Controls the time of day".to_string()
    }

    fn get_usage(&self) -> String {
        "/time set <day|night|dawn|dusk|noon|midnight>".to_string()
    }
}

pub struct WeatherCommand;
impl WeatherCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for WeatherCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /weather <clear|rain|thunder|snow>".to_string(),
                affected_systems: vec![],
            };
        }

        let weather = args[0].as_str();
        match weather {
            "clear" | "rain" | "thunder" | "snow" => {
                CommandResult {
                    success: true,
                    message: format!("Weather set to {}", weather),
                    affected_systems: vec!["weather".to_string()],
                }
            }
            _ => CommandResult {
                success: false,
                message: "Invalid weather type. Use: clear, rain, thunder, or snow".to_string(),
                affected_systems: vec![],
            },
        }
    }

    fn get_help(&self) -> String {
        "Controls the weather".to_string()
    }

    fn get_usage(&self) -> String {
        "/weather <clear|rain|thunder|snow>".to_string()
    }
}

pub struct ClearCommand;
impl ClearCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for ClearCommand {
    fn execute(&self, _args: &[String]) -> CommandResult {
        CommandResult {
            success: true,
            message: "Chat cleared.".to_string(),
            affected_systems: vec!["chat".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Clears the chat history".to_string()
    }

    fn get_usage(&self) -> String {
        "/clear".to_string()
    }
}

pub struct SeedCommand;
impl SeedCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for SeedCommand {
    fn execute(&self, _args: &[String]) -> CommandResult {
        // This would need access to the world to get the actual seed
        CommandResult {
            success: true,
            message: "World seed: 12345".to_string(), // Placeholder
            affected_systems: vec![],
        }
    }

    fn get_help(&self) -> String {
        "Shows the world seed".to_string()
    }

    fn get_usage(&self) -> String {
        "/seed".to_string()
    }
}

pub struct TeleportCommand;
impl TeleportCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for TeleportCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.len() != 3 {
            return CommandResult {
                success: false,
                message: "Usage: /tp <x> <y> <z>".to_string(),
                affected_systems: vec![],
            };
        }

        let x: f32 = match args[0].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid X coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        let y: f32 = match args[1].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid Y coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        let z: f32 = match args[2].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid Z coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        CommandResult {
            success: true,
            message: format!("Teleported to ({}, {}, {})", x, y, z),
            affected_systems: vec!["player".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Teleports to coordinates".to_string()
    }

    fn get_usage(&self) -> String {
        "/tp <x> <y> <z>".to_string()
    }
}

pub struct GamemodeCommand;
impl GamemodeCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for GamemodeCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /gamemode <0|1|2>".to_string(),
                affected_systems: vec![],
            };
        }

        let mode = match args[0].as_str() {
            "0" | "survival" => "Survival",
            "1" | "creative" => "Creative",
            "2" | "adventure" => "Adventure",
            _ => {
                return CommandResult {
                    success: false,
                    message: "Invalid game mode. Use: 0 (survival), 1 (creative), or 2 (adventure)".to_string(),
                    affected_systems: vec![],
                };
            }
        };

        CommandResult {
            success: true,
            message: format!("Game mode set to {}", mode),
            affected_systems: vec!["player".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Changes the game mode".to_string()
    }

    fn get_usage(&self) -> String {
        "/gamemode <0|1|2>".to_string()
    }
}

pub struct GiveCommand;
impl GiveCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for GiveCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /give <item> [count]".to_string(),
                affected_systems: vec![],
            };
        }

        let item = &args[0];
        let count = if args.len() > 1 {
            match args[1].parse::<u32>() {
                Ok(val) => val,
                Err(_) => return CommandResult {
                    success: false,
                    message: "Invalid count".to_string(),
                    affected_systems: vec![],
                },
            }
        } else {
            1
        };

        CommandResult {
            success: true,
            message: format!("Gave {} x{} to player", item, count),
            affected_systems: vec!["player".to_string(), "inventory".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Gives items to player".to_string()
    }

    fn get_usage(&self) -> String {
        "/give <item> [count]".to_string()
    }
}

pub struct SpawnCommand;
impl SpawnCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for SpawnCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /spawn <mob> [count]".to_string(),
                affected_systems: vec![],
            };
        }

        let mob = &args[0];
        let count = if args.len() > 1 {
            match args[1].parse::<u32>() {
                Ok(val) => val,
                Err(_) => return CommandResult {
                    success: false,
                    message: "Invalid count".to_string(),
                    affected_systems: vec![],
                },
            }
        } else {
            1
        };

        CommandResult {
            success: true,
            message: format!("Spawned {} x{}", mob, count),
            affected_systems: vec!["world".to_string(), "mobs".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Spawns mobs".to_string()
    }

    fn get_usage(&self) -> String {
        "/spawn <mob> [count]".to_string()
    }
}

pub struct KillCommand;
impl KillCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for KillCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        let target = if args.is_empty() {
            "player"
        } else {
            &args[0]
        };

        match target {
            "player" => CommandResult {
                success: true,
                message: "Player killed".to_string(),
                affected_systems: vec!["player".to_string()],
            },
            "all" => CommandResult {
                success: true,
                message: "All mobs killed".to_string(),
                affected_systems: vec!["world".to_string(), "mobs".to_string()],
            },
            _ => CommandResult {
                success: false,
                message: "Invalid target. Use: player or all".to_string(),
                affected_systems: vec![],
            },
        }
    }

    fn get_help(&self) -> String {
        "Kills entities".to_string()
    }

    fn get_usage(&self) -> String {
        "/kill [player|all]".to_string()
    }
}
// Removed duplicate imports

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_type: BlockType,
    pub count: u8,
    pub durability: Option<u16>, // For tools and armor
}

impl ItemStack {
    pub fn new(item_type: BlockType, count: u8) -> Self {
        Self {
            item_type,
            count,
            durability: None,
        }
    }

    pub fn with_durability(item_type: BlockType, count: u8, durability: u16) -> Self {
        Self {
            item_type,
            count,
            durability: Some(durability),
        }
    }

    pub fn is_stackable(&self) -> bool {
        // Tools and armor are not stackable
        !self.item_type.is_tool() && self.count < 64
    }

    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.item_type == other.item_type && 
        self.is_stackable() && 
        other.is_stackable() &&
        self.durability.is_none() && other.durability.is_none()
    }

    pub fn merge(&mut self, other: &mut ItemStack) -> bool {
        if !self.can_merge_with(other) {
            return false;
        }

        let available_space = 64 - self.count;
        let transfer_amount = other.count.min(available_space);

        self.count += transfer_amount;
        other.count -= transfer_amount;

        if other.count == 0 {
            *other = ItemStack::new(BlockType::Air, 0);
        }

        true
    }

    pub fn split(&mut self, amount: u8) -> Option<ItemStack> {
        if amount == 0 || amount > self.count {
            return None;
        }

        let split_stack = ItemStack::new(self.item_type, amount);
        self.count -= amount;

        if self.count == 0 {
            *self = ItemStack::new(BlockType::Air, 0);
        }

        Some(split_stack)
    }

    pub fn is_empty(&self) -> bool {
        self.item_type == BlockType::Air || self.count == 0
    }

    pub fn get_max_stack_size(&self) -> u8 {
        if self.item_type.is_tool() {
            1
        } else {
            64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub hotbar_slots: usize,
    pub main_slots: usize,
}

impl Inventory {
    pub fn new(hotbar_slots: usize, main_slots: usize) -> Self {
        let total_slots = hotbar_slots + main_slots;
        let mut slots = Vec::with_capacity(total_slots);
        
        for _ in 0..total_slots {
            slots.push(None);
        }

        Self {
            slots,
            hotbar_slots,
            main_slots,
        }
    }

    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        if index < self.slots.len() {
            self.slots[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Option<ItemStack>> {
        if index < self.slots.len() {
            Some(&mut self.slots[index])
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) -> bool {
        if index < self.slots.len() {
            self.slots[index] = stack;
            true
        } else {
            false
        }
    }

    pub fn add_item(&mut self, mut item: ItemStack) -> bool {
        // Try to stack with existing items first
        for i in 0..self.slots.len() {
            if let Some(ref mut existing) = self.slots[i] {
                if existing.can_merge_with(&item) {
                    existing.merge(&mut item);
                    if item.is_empty() {
                        return true;
                    }
                }
            }
        }

        // Find empty slot
        for i in 0..self.slots.len() {
            if self.slots[i].is_none() || self.slots[i].as_ref().unwrap().is_empty() {
                self.slots[i] = Some(item);
                return true;
            }
        }

        false
    }

    pub fn remove_item(&mut self, index: usize, amount: u8) -> Option<ItemStack> {
        if let Some(ref mut stack) = self.slots[index] {
            stack.split(amount)
        } else {
            None
        }
    }

    pub fn swap_slots(&mut self, index1: usize, index2: usize) -> bool {
        if index1 < self.slots.len() && index2 < self.slots.len() {
            self.slots.swap(index1, index2);
            true
        } else {
            false
        }
    }

    pub fn get_first_empty_slot(&self) -> Option<usize> {
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_none() || slot.as_ref().unwrap().is_empty() {
                return Some(i);
            }
        }
        None
    }

    pub fn count_items(&self, item_type: BlockType) -> u32 {
        let mut count = 0;
        for slot in &self.slots {
            if let Some(stack) = slot {
                if stack.item_type == item_type {
                    count += stack.count as u32;
                }
            }
        }
        count
    }

    pub fn has_item(&self, item_type: BlockType) -> bool {
        self.count_items(item_type) > 0
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragOperation {
    pub source_slot: usize,
    pub source_stack: ItemStack,
    pub drag_slots: Vec<usize>,
    pub split_mode: SplitMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitMode {
    None,
    Even,
    Half,
    Custom(u8),
}

pub struct InventoryDragHandler {
    pub current_operation: Option<DragOperation>,
    pub drag_start_slot: Option<usize>,
    pub is_dragging: bool,
}

impl InventoryDragHandler {
    pub fn new() -> Self {
        Self {
            current_operation: None,
            drag_start_slot: None,
            is_dragging: false,
        }
    }

    pub fn start_drag(&mut self, inventory: &mut Inventory, slot_index: usize) -> bool {
        if let Some(stack) = inventory.get_slot(slot_index) {
            if !stack.is_empty() {
                self.drag_start_slot = Some(slot_index);
                self.is_dragging = true;
                self.current_operation = Some(DragOperation {
                    source_slot: slot_index,
                    source_stack: *stack,
                    drag_slots: vec![slot_index],
                    split_mode: SplitMode::None,
                });
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn update_drag(&mut self, inventory: &mut Inventory, slot_index: usize, split_mode: SplitMode) -> bool {
        if !self.is_dragging || self.drag_start_slot.is_none() {
            return false;
        }

        let source_slot = self.drag_start_slot.unwrap();
        if source_slot == slot_index {
            return false;
        }

        if let Some(ref mut operation) = self.current_operation {
            // Check if this slot is already in the drag path
            if operation.drag_slots.contains(&slot_index) {
                return false;
            }

            // Try to add this slot to the drag operation
            if let Some(source_stack) = inventory.get_slot(source_slot) {
                if let Some(target_stack) = inventory.get_slot(slot_index) {
                    // Check if we can merge
                    if source_stack.can_merge_with(target_stack) {
                        operation.drag_slots.push(slot_index);
                        operation.split_mode = split_mode;
                        return true;
                    }
                } else {
                    // Empty slot - can place item here
                    operation.drag_slots.push(slot_index);
                    operation.split_mode = split_mode;
                    return true;
                }
            }
        }

        false
    }

    pub fn end_drag(&mut self, inventory: &mut Inventory) -> bool {
        if !self.is_dragging || self.current_operation.is_none() {
            return false;
        }

        let operation = self.current_operation.take().unwrap();
        let source_slot = operation.source_slot;

        if let Some(source_stack) = inventory.remove_item(source_slot, operation.source_stack.count) {
            let mut remaining_stack = source_stack;

            for &target_slot in &operation.drag_slots {
                if target_slot == source_slot {
                    continue;
                }

                let split_amount = match operation.split_mode {
                    SplitMode::None => remaining_stack.count,
                    SplitMode::Even => (remaining_stack.count / operation.drag_slots.len() as u8).max(1),
                    SplitMode::Half => remaining_stack.count / 2,
                    SplitMode::Custom(amount) => amount.min(remaining_stack.count),
                };

                if let Some(mut split_stack) = remaining_stack.split(split_amount) {
                    if let Some(target_stack) = inventory.get_slot_mut(target_slot) {
                        if target_stack.is_none() || target_stack.as_ref().unwrap().is_empty() {
                            *target_stack = Some(split_stack);
                        } else if let Some(ref mut existing) = target_stack {
                            if existing.can_merge_with(&split_stack) {
                                existing.merge(&mut split_stack);
                            }
                        }
                    }
                }

                if remaining_stack.is_empty() {
                    break;
                }
            }

            // Put remaining items back in source slot
            if !remaining_stack.is_empty() {
                inventory.set_slot(source_slot, Some(remaining_stack));
            }
        }

        self.is_dragging = false;
        self.drag_start_slot = None;
        true
    }

    pub fn cancel_drag(&mut self) {
        self.current_operation = None;
        self.is_dragging = false;
        self.drag_start_slot = None;
    }

    pub fn get_drag_preview(&self, _inventory: &Inventory) -> Vec<(usize, ItemStack)> {
        let mut preview = Vec::new();

        if let Some(ref operation) = self.current_operation {
            let total_slots = operation.drag_slots.len();
            
            for (i, &slot_index) in operation.drag_slots.iter().enumerate() {
                let split_amount = match operation.split_mode {
                    SplitMode::None => {
                        if i == 0 {
                            operation.source_stack.count
                        } else {
                            0
                        }
                    }
                    SplitMode::Even => (operation.source_stack.count / total_slots as u8).max(1),
                    SplitMode::Half => operation.source_stack.count / 2,
                    SplitMode::Custom(amount) => amount.min(operation.source_stack.count),
                };

                if split_amount > 0 {
                    preview.push((slot_index, ItemStack::new(operation.source_stack.item_type, split_amount)));
                }
            }
        }

        preview
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingGrid {
    pub slots: Vec<Option<ItemStack>>,
    pub width: usize,
    pub height: usize,
}

impl CraftingGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let total_slots = width * height;
        let mut slots = Vec::with_capacity(total_slots);
        
        for _ in 0..total_slots {
            slots.push(None);
        }

        Self {
            slots,
            width,
            height,
        }
    }

    pub fn get_slot(&self, x: usize, y: usize) -> Option<&ItemStack> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.slots[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_slot_mut(&mut self, x: usize, y: usize) -> Option<&mut Option<ItemStack>> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(&mut self.slots[index])
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, x: usize, y: usize, stack: Option<ItemStack>) -> bool {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.slots[index] = stack;
            true
        } else {
            false
        }
    }

    pub fn get_index_from_coords(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_coords_from_index(&self, index: usize) -> Option<(usize, usize)> {
        if index < self.slots.len() {
            Some((index % self.width, index / self.width))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    pub fn apply_drag_split(&mut self, drag_handler: &InventoryDragHandler) -> bool {
        if let Some(ref operation) = drag_handler.current_operation {
            let total_slots = operation.drag_slots.len();
            
            for (i, &slot_index) in operation.drag_slots.iter().enumerate() {
                if let Some((x, y)) = self.get_coords_from_index(slot_index) {
                    let split_amount = match operation.split_mode {
                        SplitMode::None => {
                            if i == 0 {
                                operation.source_stack.count
                            } else {
                                0
                            }
                        }
                        SplitMode::Even => (operation.source_stack.count / total_slots as u8).max(1),
                        SplitMode::Half => operation.source_stack.count / 2,
                        SplitMode::Custom(amount) => amount.min(operation.source_stack.count),
                    };

                    if split_amount > 0 {
                        let split_stack = ItemStack::new(operation.source_stack.item_type, split_amount);
                        self.set_slot(x, y, Some(split_stack));
                    }
                }
            }
            true
        } else {
            false
        }
    }
}
// Removed duplicate imports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_name: String,
    pub skin: PlayerSkin,
    pub nametag_color: [u8; 3], // RGB color
    pub show_nametag: bool,
    pub custom_cape: Option<String>, // Cape texture name
    pub achievements: Vec<String>,
    pub statistics: PlayerStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSkin {
    pub skin_type: SkinType,
    pub base_color: [u8; 3], // RGB
    pub hair_color: [u8; 3], // RGB
    pub eye_color: [u8; 3], // RGB
    pub shirt_color: [u8; 3], // RGB
    pub pants_color: [u8; 3], // RGB
    pub shoes_color: [u8; 3], // RGB
    pub accessories: Vec<String>, // Hat, glasses, etc.
    pub custom_texture: Option<String>, // Custom texture path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkinType {
    Steve, // Male
    Alex, // Female
    Custom, // Custom model
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    pub blocks_placed: u64,
    pub blocks_broken: u64,
    pub mobs_killed: u64,
    pub deaths: u64,
    pub distance_walked: f64,
    pub time_played: f64,
    pub items_crafted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nametag {
    pub player_name: String,
    pub position: Vec3,
    pub color: [u8; 3],
    pub visible: bool,
    pub offset_y: f32, // Height above player head
}

pub struct PlayerCustomizationSystem {
    pub profiles: HashMap<String, PlayerProfile>,
    pub current_profile: Option<String>,
    pub available_skins: Vec<SkinType>,
    pub available_colors: Vec<[u8; 3]>,
    pub nametags: Vec<Nametag>,
}

impl PlayerCustomizationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            profiles: HashMap::new(),
            current_profile: None,
            available_skins: vec![SkinType::Steve, SkinType::Alex],
            available_colors: vec![
                [255, 255, 255], // White
                [0, 0, 0], // Black
                [255, 0, 0], // Red
                [0, 255, 0], // Green
                [0, 0, 255], // Blue
                [255, 255, 0], // Yellow
                [255, 0, 255], // Magenta
                [0, 255, 255], // Cyan
                [128, 128, 128], // Gray
                [165, 42, 42], // Brown
                [255, 165, 0], // Orange
                [128, 0, 128], // Purple
                [0, 128, 128], // Teal
            ],
            nametags: Vec::new(),
        };

        // Create default profile
        let default_profile = PlayerProfile {
            player_name: "Player".to_string(),
            skin: PlayerSkin {
                skin_type: SkinType::Steve,
                base_color: [255, 220, 177], // Light skin tone
                hair_color: [139, 69, 19], // Brown
                eye_color: [0, 0, 0], // Black
                shirt_color: [0, 0, 255], // Blue shirt
                pants_color: [0, 0, 139], // Dark blue pants
                shoes_color: [0, 0, 0], // Black shoes
                accessories: Vec::new(),
                custom_texture: None,
            },
            nametag_color: [255, 255, 255], // White
            show_nametag: true,
            custom_cape: None,
            achievements: Vec::new(),
            statistics: PlayerStatistics {
                blocks_placed: 0,
                blocks_broken: 0,
                mobs_killed: 0,
                deaths: 0,
                distance_walked: 0.0,
                time_played: 0.0,
                items_crafted: 0,
            },
        };

        system.profiles.insert("default".to_string(), default_profile);
        system.current_profile = Some("default".to_string());

        system
    }

    pub fn create_profile(&mut self, name: &str, profile: PlayerProfile) -> Result<(), String> {
        if self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' already exists", name));
        }

        self.profiles.insert(name.to_string(), profile);
        Ok(())
    }

    pub fn get_current_profile(&self) -> Option<&PlayerProfile> {
        if let Some(ref profile_name) = self.current_profile {
            self.profiles.get(profile_name)
        } else {
            None
        }
    }

    pub fn get_current_profile_mut(&mut self) -> Option<&mut PlayerProfile> {
        if let Some(ref profile_name) = self.current_profile {
            self.profiles.get_mut(profile_name)
        } else {
            None
        }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Result<(), String> {
        if !self.profiles.contains_key(profile_name) {
            return Err(format!("Profile '{}' does not exist", profile_name));
        }

        self.current_profile = Some(profile_name.to_string());
        Ok(())
    }

    pub fn update_player_name(&mut self, name: &str) -> Result<(), String> {
        if name.is_empty() || name.len() > 16 {
            return Err("Name must be between 1 and 16 characters".to_string());
        }

        if let Some(profile) = self.get_current_profile_mut() {
            profile.player_name = name.to_string();
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_skin_color(&mut self, color_type: SkinColorType, color: [u8; 3]) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            match color_type {
                SkinColorType::Base => profile.skin.base_color = color,
                SkinColorType::Hair => profile.skin.hair_color = color,
                SkinColorType::Eyes => profile.skin.eye_color = color,
                SkinColorType::Shirt => profile.skin.shirt_color = color,
                SkinColorType::Pants => profile.skin.pants_color = color,
                SkinColorType::Shoes => profile.skin.shoes_color = color,
            }
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_nametag_color(&mut self, color: [u8; 3]) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.nametag_color = color;
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn toggle_nametag(&mut self) -> Result<bool, String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.show_nametag = !profile.show_nametag;
            Ok(profile.show_nametag)
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn add_accessory(&mut self, accessory: String) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            if !profile.skin.accessories.contains(&accessory) {
                profile.skin.accessories.push(accessory);
            }
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn remove_accessory(&mut self, accessory: &str) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.skin.accessories.retain(|a| a != accessory);
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_nametags(&mut self, player_positions: &[(String, Vec3)]) {
        self.nametags.clear();
        
        for (player_name, position) in player_positions {
            if let Some(profile) = self.profiles.get(player_name) {
                if profile.show_nametag {
                    let nametag = Nametag {
                        player_name: profile.player_name.clone(),
                        position: *position + Vec3::new(0.0, 1.8, 0.0), // Above player head
                        color: profile.nametag_color,
                        visible: true,
                        offset_y: 0.3,
                    };
                    self.nametags.push(nametag);
                }
            }
        }
    }

    pub fn get_nametag_at_position(&self, position: Vec3, view_direction: Vec3, max_distance: f32) -> Option<&Nametag> {
        for nametag in &self.nametags {
            let distance = (nametag.position - position).length();
            if distance <= max_distance {
                // Check if player is looking at this nametag
                let to_nametag = nametag.position - position;
                let dot = view_direction.dot(to_nametag.normalize());
                if dot > 0.7 { // Within ~45 degree cone
                    return Some(nametag);
                }
            }
        }
        None
    }

    pub fn get_all_nametags_in_range(&self, position: Vec3, max_distance: f32) -> Vec<&Nametag> {
        self.nametags
            .iter()
            .filter(|nametag| (nametag.position - position).length() <= max_distance)
            .collect()
    }

    pub fn save_profiles(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.profiles)
    }

    pub fn load_profiles(&mut self, json_data: &str) -> Result<(), serde_json::Error> {
        self.profiles = serde_json::from_str(json_data)?;
        Ok(())
    }

    pub fn get_available_colors(&self) -> &[[u8; 3]] {
        &self.available_colors
    }

    pub fn get_available_skins(&self) -> &[SkinType] {
        &self.available_skins
    }

    pub fn get_available_accessories(&self) -> Vec<String> {
        vec![
            "None".to_string(),
            "Baseball Cap".to_string(),
            "Crown".to_string(),
            "Glasses".to_string(),
            "Headband".to_string(),
            "Mining Helmet".to_string(),
            "Sunglasses".to_string(),
            "Top Hat".to_string(),
            "Witch Hat".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SkinColorType {
    Base,
    Hair,
    Eyes,
    Shirt,
    Pants,
    Shoes,
}

impl PlayerCustomizationSystem {
    pub fn get_skin_texture_coords(&self, profile: &PlayerProfile) -> [u32; 6] {
        // Return texture coordinates for different skin parts
        // This would be used by the renderer to draw the custom skin
        match profile.skin.skin_type {
            SkinType::Steve => [0, 1, 2, 3, 4, 5], // Default Steve textures
            SkinType::Alex => [6, 7, 8, 9, 10, 11], // Default Alex textures
            SkinType::Custom => [12, 13, 14, 15, 16, 17], // Custom texture
        }
    }

    pub fn get_skin_colors(&self, profile: &PlayerProfile) -> [[u8; 3]; 6] {
        [
            profile.skin.base_color,
            profile.skin.hair_color,
            profile.skin.eye_color,
            profile.skin.shirt_color,
            profile.skin.pants_color,
            profile.skin.shoes_color,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinPreset {
    pub name: String,
    pub skin: PlayerSkin,
    pub description: String,
}

impl PlayerCustomizationSystem {
    pub fn get_default_presets(&self) -> Vec<SkinPreset> {
        vec![
            SkinPreset {
                name: "Default Steve".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Steve,
                    base_color: [255, 220, 177],
                    hair_color: [139, 69, 19],
                    eye_color: [0, 0, 0],
                    shirt_color: [0, 0, 255],
                    pants_color: [0, 0, 139],
                    shoes_color: [0, 0, 0],
                    accessories: Vec::new(),
                    custom_texture: None,
                },
                description: "Classic Steve skin".to_string(),
            },
            SkinPreset {
                name: "Default Alex".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Alex,
                    base_color: [255, 220, 177],
                    hair_color: [255, 140, 0], // Blonde hair
                    eye_color: [0, 100, 0], // Green eyes
                    shirt_color: [255, 0, 0], // Red shirt
                    pants_color: [128, 0, 128], // Purple pants
                    shoes_color: [139, 69, 19], // Brown shoes
                    accessories: vec!["Baseball Cap".to_string()],
                    custom_texture: None,
                },
                description: "Classic Alex skin with baseball cap".to_string(),
            },
            SkinPreset {
                name: "Miner".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Steve,
                    base_color: [139, 69, 19], // Darker skin
                    hair_color: [0, 0, 0], // Black hair
                    eye_color: [139, 69, 19], // Brown eyes
                    shirt_color: [128, 128, 128], // Gray shirt
                    pants_color: [64, 64, 64], // Dark gray pants
                    shoes_color: [0, 0, 0], // Black shoes
                    accessories: vec!["Mining Helmet".to_string()],
                    custom_texture: None,
                },
                description: "Professional miner outfit".to_string(),
            },
        ]
    }

    pub fn apply_preset(&mut self, preset_name: &str) -> Result<(), String> {
        let presets = self.get_default_presets();
        
        if let Some(preset) = presets.iter().find(|p| p.name == preset_name) {
            if let Some(profile) = self.get_current_profile_mut() {
                profile.skin = preset.skin.clone();
                Ok(())
            } else {
                Err("No profile selected".to_string())
            }
        } else {
            Err(format!("Preset '{}' not found", preset_name))
        }
    }
}
