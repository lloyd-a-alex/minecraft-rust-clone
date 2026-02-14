//! DIABOLICAL UI SYSTEM - Advanced User Interface Management
//! 
//! This module provides comprehensive UI management with support for:
//! - Dynamic menu systems with animations
//! - Advanced HUD with real-time information
//! - Interactive crafting interfaces
//! - Particle effects and visual feedback
//! - Responsive design for different screen sizes

use glam::Vec3;
use crate::world::{BlockType, BlockPos};
use crate::player::Player;
use crate::renderer::Renderer;
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

    pub fn update(&mut self, dt: f32, player: &Player, world: &crate::world::World) {
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
