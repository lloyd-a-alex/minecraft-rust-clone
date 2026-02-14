//! DIABOLICAL MINECRAFT RUST CLONE - Main Library
//! 
//! This is the main library for the Minecraft Rust clone, containing all the
//! core modules and systems that make up the game.

// Shared types used across modules
#[derive(PartialEq)]
pub enum GameState { Loading, Menu, Multiplayer, Playing, Settings }

pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
impl Rect { 
    pub fn contains(&self, nx: f32, ny: f32) -> bool { 
        nx >= self.x - self.w/2.0 && nx <= self.x + self.w/2.0 && ny >= self.y - self.h/2.0 && ny <= self.y + self.h/2.0 
    } 
}

#[derive(Clone)]
pub enum MenuAction { 
    Singleplayer, 
    Host, 
    JoinMenu, 
    JoinAddr(String), 
    Stress, 
    Resume, 
    Quit,
    Settings
}

pub struct MenuButton { 
    pub rect: Rect, 
    pub text: String, 
    pub action: MenuAction, 
    pub hovered: bool 
}

pub struct MainMenu { 
    pub buttons: Vec<MenuButton> 
}

pub struct SettingsMenu {
    pub buttons: Vec<MenuButton>,
    pub settings_values: SettingsValues,
}

pub struct SettingsValues {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub render_distance: u32,
    pub fov: f32,
    pub max_fps: u32,
    pub shader_type: ShaderType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Classic,      // minecraft_shaders.wgsl
    Traditional,   // traditional_shaders.wgsl
    Basic,        // shader.wgsl
}

impl ShaderType {
    pub fn get_file_name(&self) -> &'static str {
        match self {
            ShaderType::Classic => "minecraft_shaders.wgsl",
            ShaderType::Traditional => "traditional_shaders.wgsl",
            ShaderType::Basic => "shader.wgsl",
        }
    }
    
    pub fn get_display_name(&self) -> &'static str {
        match self {
            ShaderType::Classic => "Classic Minecraft",
            ShaderType::Traditional => "Traditional Enhanced",
            ShaderType::Basic => "Basic Renderer",
        }
    }
    
    pub fn get_description(&self) -> &'static str {
        match self {
            ShaderType::Classic => "Authentic Java Edition rendering with nearest-neighbor filtering",
            ShaderType::Traditional => "Enhanced rendering with material properties and advanced lighting",
            ShaderType::Basic => "Simple, fast rendering for maximum performance",
        }
    }
}

impl SettingsValues {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            render_distance: 8,
            fov: 90.0,
            max_fps: 60,
            shader_type: ShaderType::Classic,
        }
    }
    
    pub fn cycle_shader(&mut self) {
        self.shader_type = match self.shader_type {
            ShaderType::Classic => ShaderType::Traditional,
            ShaderType::Traditional => ShaderType::Basic,
            ShaderType::Basic => ShaderType::Classic,
        };
    }
    
    pub fn adjust_shader(&mut self, direction: i32) {
        let shaders = [ShaderType::Classic, ShaderType::Traditional, ShaderType::Basic];
        let current_index = shaders.iter().position(|&s| s == self.shader_type).unwrap_or(0);
        let new_index = (current_index as i32 + direction).rem_euclid(shaders.len() as i32) as usize;
        self.shader_type = shaders[new_index];
    }
}

impl SettingsMenu {
    pub fn new() -> Self {
        let mut b = Vec::new();
        let w = 0.6; let h = 0.08; let g = 0.04; let sy = 0.2;
        
        b.push(MenuButton{rect:Rect{x:0.0,y:sy,w,h}, text:"MASTER VOLUME".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g),w,h}, text:"MUSIC VOLUME".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*2.0,w,h}, text:"SFX VOLUME".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*3.0,w,h}, text:"RENDER DISTANCE".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*4.0,w,h}, text:"FIELD OF VIEW".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*5.0,w,h}, text:"MAX FPS".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*6.0,w,h}, text:"SHADER TYPE".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*8.0,w,h}, text:"BACK".to_string(), action:MenuAction::Quit, hovered:false});
        
        SettingsMenu {
            buttons: b,
            settings_values: SettingsValues::new(),
        }
    }
}

impl MainMenu {
    pub fn new_main() -> Self {
        let mut b = Vec::new();
        let w = 0.8; let h = 0.12; let g = 0.05; let sy = 0.3; 
        b.push(MenuButton{rect:Rect{x:0.0,y:sy,w,h}, text:"SINGLEPLAYER".to_string(), action:MenuAction::Singleplayer, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g),w,h}, text:"MULTIPLAYER".to_string(), action:MenuAction::JoinMenu, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*2.0,w,h}, text:"HOST WORLD".to_string(), action:MenuAction::Host, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*3.0,w,h}, text:"STRESS TEST".to_string(), action:MenuAction::Stress, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*4.0,w,h}, text:"SETTINGS".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*5.5,w,h}, text:"QUIT".to_string(), action:MenuAction::Quit, hovered:false});
        MainMenu { buttons: b }
    }

    pub fn new_pause() -> Self {
        let mut b = Vec::new();
        // Professional centered layout for pause menu - CORRECT POSITIONS
        let panel_x = 0.0;
        let panel_y = 0.0;
        let button_width = 0.4;
        let button_height = 0.06;
        let button_spacing = 0.08;
        let start_y = panel_y - button_spacing/2.0;
        
        // Buttons are rendered at (panel_x - button_width/2.0, button_y - button_height/2.0)
        // So rect should be centered at (panel_x, button_y)
        b.push(MenuButton{rect:Rect{x:panel_x,y:start_y,w:button_width,h:button_height}, text:"RESUME GAME".to_string(), action:MenuAction::Resume, hovered:false});
        b.push(MenuButton{rect:Rect{x:panel_x,y:start_y-button_spacing,w:button_width,h:button_height}, text:"SETTINGS".to_string(), action:MenuAction::Settings, hovered:false});
        b.push(MenuButton{rect:Rect{x:panel_x,y:start_y-button_spacing*2.0,w:button_width,h:button_height}, text:"QUIT TO MENU".to_string(), action:MenuAction::Quit, hovered:false});
        MainMenu { buttons: b }
    }
}

// Audio system
pub struct AudioSystem {
    _stream: Option<rodio::OutputStream>,
    #[allow(dead_code)]
    stream_handle: Option<rodio::OutputStreamHandle>,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

impl AudioSystem {
    pub fn new() -> Self {
        let (stream, handle) = match rodio::OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };
        Self { 
            _stream: stream, 
            stream_handle: handle,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn play(&self, _sound_type: &str, _in_cave: bool) {
        // Placeholder implementation
    }

    pub fn play_step(&self, _category: &str, _variant: usize, _in_cave: bool) {
        // Placeholder implementation
    }
}

pub struct Hotbar { 
    pub slots: [Option<(crate::engine::BlockType, u32)>; 9], 
    pub selected_slot: usize 
}

impl Hotbar { 
    pub fn new() -> Self { 
        Self { 
            slots: [None; 9], 
            selected_slot: 0 
        } 
    } 
}

pub mod engine;
pub mod environment;
pub mod graphics;
pub mod interface;
pub mod configuration;
pub mod network;
pub mod resources;
pub mod utils;

// Re-export commonly used types for convenience
pub use crate::engine::{Player, World, BlockPos, BlockType};
pub use crate::graphics::Renderer;
pub use crate::resources::{ResourceTracker, track_chunk_usage, cleanup_if_needed};
pub use crate::network::NetworkManager;
pub use crate::environment::{WeatherSystem, WeatherType};
pub use crate::configuration::{ConfigManager, GameConfig};
// pub use crate::{MainMenu, SettingsMenu, SettingsValues};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
