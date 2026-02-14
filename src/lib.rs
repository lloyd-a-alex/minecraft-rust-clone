//! DIABOLICAL MINECRAFT RUST CLONE - Main Library
//! 
//! This is the main library for the Minecraft Rust clone, containing all the
//! core modules and systems that make up the game.

// Shared types used across modules
#[derive(PartialEq)]
pub enum GameState { Loading, Menu, Multiplayer, Playing }

pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
impl Rect { 
    fn contains(&self, nx: f32, ny: f32) -> bool { 
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
    Quit 
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

impl MainMenu {
    pub fn new_main() -> Self {
        let mut b = Vec::new();
        let w = 0.8; let h = 0.12; let g = 0.05; let sy = 0.3; 
        b.push(MenuButton{rect:Rect{x:0.0,y:sy,w,h}, text:"SINGLEPLAYER".to_string(), action:MenuAction::Singleplayer, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g),w,h}, text:"MULTIPLAYER".to_string(), action:MenuAction::JoinMenu, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*2.0,w,h}, text:"HOST WORLD".to_string(), action:MenuAction::Host, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*3.0,w,h}, text:"STRESS TEST".to_string(), action:MenuAction::Stress, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*4.5,w,h}, text:"QUIT".to_string(), action:MenuAction::Quit, hovered:false});
        MainMenu { buttons: b }
    }

    pub fn new_pause() -> Self {
        let mut b = Vec::new();
        let w = 0.8; let h = 0.12; let g = 0.05; let sy = 0.1;
        b.push(MenuButton{rect:Rect{x:0.0,y:sy,w,h}, text:"RESUME GAME".to_string(), action:MenuAction::Resume, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*1.5,w,h}, text:"QUIT TO MENU".to_string(), action:MenuAction::Quit, hovered:false});
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

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
