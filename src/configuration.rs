// DIABOLICAL CONFIGURATION SYSTEM - Centralized Settings Management
// 
// This module provides comprehensive configuration management including:
// - Game settings and preferences
// - Performance tuning parameters
// - Difficulty settings
// - Graphics configuration
// - Audio settings
// - Control mappings
// - Network settings

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// DIABOLICAL Game Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub graphics: GraphicsConfig,
    pub audio: AudioConfig,
    pub controls: ControlsConfig,
    pub gameplay: GameplayConfig,
    pub network: NetworkConfig,
    pub performance: PerformanceConfig,
    pub ui: UIConfig,
    pub debug: DebugConfig,
}

/// DIABOLICAL Graphics Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub vsync: bool,
    pub fov: f32,
    pub render_distance: u32,
    pub max_fps: u32,
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
    pub particle_quality: ParticleQuality,
    pub anti_aliasing: AntiAliasing,
    pub anisotropic_filtering: u8,
    pub brightness: f32,
    pub gamma: f32,
    pub weather_effects: bool,
    pub ambient_occlusion: bool,
    pub bloom: bool,
    pub motion_blur: bool,
    pub depth_of_field: bool,
    pub screen_space_reflections: bool,
    pub volumetric_fog: bool,
    pub dynamic_lighting: bool,
    pub ray_tracing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntiAliasing {
    Off,
    FXAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
    TAA,
}

/// DIABOLICAL Audio Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
    pub voice_volume: f32,
    pub ui_volume: f32,
    pub enable_3d_audio: bool,
    pub audio_device: Option<String>,
    pub audio_backend: AudioBackend,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub max_channels: u32,
    pub compression_enabled: bool,
    pub reverb_enabled: bool,
    pub hrtf_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioBackend {
    Default,
    WASAPI,
    DirectSound,
    ALSA,
    PulseAudio,
    CoreAudio,
}

/// DIABOLICAL Controls Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsConfig {
    pub keybindings: HashMap<String, KeyCode>,
    pub mouse_sensitivity: f32,
    pub mouse_acceleration: bool,
    pub invert_y: bool,
    pub raw_input: bool,
    pub controller_sensitivity: f32,
    pub controller_deadzone: f32,
    pub vibration_enabled: bool,
    pub auto_sprint: bool,
    pub auto_jump: bool,
    pub double_tap_sprint: bool,
}

/// DIABOLICAL Gameplay Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
    pub difficulty: Difficulty,
    pub game_mode: GameMode,
    pub hardcore: bool,
    pub peaceful: bool,
    pub pvp_enabled: bool,
    pub fire_spread: bool,
    pub mob_griefing: bool,
    pub keep_inventory: bool,
    pub natural_regeneration: bool,
    pub daylight_cycle: bool,
    pub weather_cycle: bool,
    pub mob_spawning: bool,
    pub animal_spawning: bool,
    pub villager_trading: bool,
    pub respawn_anchor: bool,
    pub nether_portal: bool,
    pub end_portal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
    Hardcore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

/// DIABOLICAL Network Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub server_port: u16,
    pub max_players: u32,
    pub server_name: String,
    pub motd: String,
    pub online_mode: bool,
    pub pvp: bool,
    pub whitelist: bool,
    pub spawn_protection: u32,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub compression_threshold: i32,
    pub compression_level: u8,
    pub network_timeout: u32,
    pub keep_alive_interval: u32,
    pub rate_limit: u32,
    pub bandwidth_limit: u32,
}

/// DIABOLICAL Performance Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_threads: usize,
    pub thread_priority: ThreadPriority,
    pub memory_limit_mb: u32,
    pub chunk_cache_size: u32,
    pub entity_render_distance: u32,
    pub particle_limit: u32,
    pub sound_channels: u32,
    pub texture_streaming: bool,
    pub chunk_streaming: bool,
    pub async_chunk_loading: bool,
    pub preloading_enabled: bool,
    pub garbage_collection: GarbageCollection,
    pub memory_optimization: bool,
    pub cpu_optimization: bool,
    pub gpu_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadPriority {
    Low,
    Normal,
    High,
    Realtime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GarbageCollection {
    Automatic,
    Manual,
    Incremental,
    Generational,
}

/// DIABOLICAL UI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub scale: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub font_family: String,
    pub theme: UITheme,
    pub show_fps: bool,
    pub show_coordinates: bool,
    pub show_biome: bool,
    pub show_time: bool,
    pub show_weather: bool,
    pub show_debug_info: bool,
    pub minimap_enabled: bool,
    pub minimap_size: f32,
    pub minimap_opacity: f32,
    pub chat_opacity: f32,
    pub inventory_opacity: f32,
    pub hotbar_opacity: f32,
    pub crosshair_style: CrosshairStyle,
    pub animation_speed: f32,
    pub particle_effects: bool,
    pub screen_shake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UITheme {
    Default,
    Dark,
    Light,
    HighContrast,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrosshairStyle {
    Default,
    Classic,
    Diabolical,
    Animated,
    Custom(String),
}

/// DIABOLICAL Debug Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub show_hitboxes: bool,
    pub show_chunk_boundaries: bool,
    pub show_entity_count: bool,
    pub show_memory_usage: bool,
    pub show_fps_graph: bool,
    pub show_network_stats: bool,
    pub show_profiling: bool,
    pub show_lighting: bool,
    pub show_collision_boxes: bool,
    pub show_pathfinding: bool,
    pub show_ai_states: bool,
    pub log_level: LogLevel,
    pub crash_reports: bool,
    pub telemetry: bool,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl GameConfig {
    pub fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            controls: ControlsConfig::default(),
            gameplay: GameplayConfig::default(),
            network: NetworkConfig::default(),
            performance: PerformanceConfig::default(),
            ui: UIConfig::default(),
            debug: DebugConfig::default(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: GameConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate graphics settings
        if self.graphics.fov < 30.0 || self.graphics.fov > 120.0 {
            return Err(ConfigError::InvalidValue("FOV must be between 30 and 120".to_string()));
        }
        
        if self.graphics.render_distance < 2 || self.graphics.render_distance > 32 {
            return Err(ConfigError::InvalidValue("Render distance must be between 2 and 32".to_string()));
        }

        // Validate audio settings
        if self.audio.master_volume < 0.0 || self.audio.master_volume > 1.0 {
            return Err(ConfigError::InvalidValue("Master volume must be between 0 and 1".to_string()));
        }

        // Validate performance settings
        if self.performance.max_threads == 0 {
            return Err(ConfigError::InvalidValue("Max threads must be at least 1".to_string()));
        }

        Ok(())
    }

    pub fn apply_graphics_settings(&self, renderer: &mut crate::renderer::Renderer) {
        // Apply graphics settings to renderer
        // This would need to be implemented with the actual renderer
        renderer.set_render_distance(self.graphics.render_distance);
        renderer.set_max_fps(self.graphics.max_fps);
        renderer.set_fov(self.graphics.fov);
    }

    pub fn apply_audio_settings(&self, audio_system: &mut crate::AudioSystem) {
        // Apply audio settings to audio system
        // This would need to be implemented with the actual audio system
        audio_system.set_master_volume(self.audio.master_volume);
        audio_system.set_music_volume(self.audio.music_volume);
        audio_system.set_sfx_volume(self.audio.sfx_volume);
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fullscreen: false,
            vsync: true,
            fov: 75.0,
            render_distance: 12,
            max_fps: 60,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::High,
            particle_quality: ParticleQuality::Medium,
            anti_aliasing: AntiAliasing::FXAA,
            anisotropic_filtering: 4,
            brightness: 1.0,
            gamma: 1.0,
            weather_effects: true,
            ambient_occlusion: true,
            bloom: true,
            motion_blur: false,
            depth_of_field: false,
            screen_space_reflections: false,
            volumetric_fog: false,
            dynamic_lighting: true,
            ray_tracing: false,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            ambient_volume: 0.6,
            voice_volume: 1.0,
            ui_volume: 0.8,
            enable_3d_audio: true,
            audio_device: None,
            audio_backend: AudioBackend::Default,
            sample_rate: 44100,
            buffer_size: 512,
            max_channels: 32,
            compression_enabled: true,
            reverb_enabled: true,
            hrtf_enabled: false,
        }
    }
}

impl Default for ControlsConfig {
    fn default() -> Self {
        let mut keybindings = HashMap::new();
        
        // Default keybindings
        keybindings.insert("move_forward".to_string(), KeyCode::W);
        keybindings.insert("move_backward".to_string(), KeyCode::S);
        keybindings.insert("move_left".to_string(), KeyCode::A);
        keybindings.insert("move_right".to_string(), KeyCode::D);
        keybindings.insert("jump".to_string(), KeyCode::Space);
        keybindings.insert("sprint".to_string(), KeyCode::LShift);
        keybindings.insert("sneak".to_string(), KeyCode::LControl);
        keybindings.insert("attack".to_string(), KeyCode::LButton);
        keybindings.insert("use".to_string(), KeyCode::RButton);
        keybindings.insert("inventory".to_string(), KeyCode::E);
        keybindings.insert("drop".to_string(), KeyCode::Q);
        keybindings.insert("chat".to_string(), KeyCode::T);
        keybindings.insert("pause".to_string(), KeyCode::Escape);

        Self {
            keybindings,
            mouse_sensitivity: 1.0,
            mouse_acceleration: false,
            invert_y: false,
            raw_input: true,
            controller_sensitivity: 1.0,
            controller_deadzone: 0.1,
            vibration_enabled: true,
            auto_sprint: false,
            auto_jump: false,
            double_tap_sprint: true,
        }
    }
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            game_mode: GameMode::Survival,
            hardcore: false,
            peaceful: false,
            pvp_enabled: true,
            fire_spread: true,
            mob_griefing: true,
            keep_inventory: false,
            natural_regeneration: true,
            daylight_cycle: true,
            weather_cycle: true,
            mob_spawning: true,
            animal_spawning: true,
            villager_trading: true,
            respawn_anchor: true,
            nether_portal: true,
            end_portal: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_port: 25565,
            max_players: 20,
            server_name: "Minecraft Server".to_string(),
            motd: "A Minecraft Server".to_string(),
            online_mode: true,
            pvp: true,
            whitelist: false,
            spawn_protection: 16,
            view_distance: 10,
            simulation_distance: 10,
            compression_threshold: 256,
            compression_level: 6,
            network_timeout: 30,
            keep_alive_interval: 15,
            rate_limit: 20,
            bandwidth_limit: 0,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get(),
            thread_priority: ThreadPriority::Normal,
            memory_limit_mb: 2048,
            chunk_cache_size: 100,
            entity_render_distance: 64,
            particle_limit: 1000,
            sound_channels: 32,
            texture_streaming: true,
            chunk_streaming: true,
            async_chunk_loading: true,
            preloading_enabled: true,
            garbage_collection: GarbageCollection::Automatic,
            memory_optimization: true,
            cpu_optimization: true,
            gpu_optimization: true,
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            opacity: 1.0,
            font_size: 16.0,
            font_family: "Arial".to_string(),
            theme: UITheme::Default,
            show_fps: false,
            show_coordinates: false,
            show_biome: false,
            show_time: false,
            show_weather: false,
            show_debug_info: false,
            minimap_enabled: false,
            minimap_size: 0.2,
            minimap_opacity: 0.8,
            chat_opacity: 0.8,
            inventory_opacity: 0.8,
            hotbar_opacity: 0.8,
            crosshair_style: CrosshairStyle::Default,
            animation_speed: 1.0,
            particle_effects: true,
            screen_shake: true,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_hitboxes: false,
            show_chunk_boundaries: false,
            show_entity_count: false,
            show_memory_usage: false,
            show_fps_graph: false,
            show_network_stats: false,
            show_profiling: false,
            show_lighting: false,
            show_collision_boxes: false,
            show_pathfinding: false,
            show_ai_states: false,
            log_level: LogLevel::Info,
            crash_reports: true,
            telemetry: false,
            debug_mode: false,
        }
    }
}

/// DIABOLICAL Configuration Error Types
#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
    InvalidValue(String),
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO Error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid Value: {}", msg),
            ConfigError::MissingField(msg) => write!(f, "Missing Field: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// DIABOLICAL Configuration Manager
pub struct ConfigManager {
    config: GameConfig,
    config_path: String,
    auto_save: bool,
}

impl ConfigManager {
    pub fn new(config_path: String) -> Self {
        let config = Self::load_or_create_default(&config_path);
        
        Self {
            config,
            config_path,
            auto_save: true,
        }
    }

    fn load_or_create_default(path: &str) -> GameConfig {
        match GameConfig::load_from_file(path) {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    log::warn!("Config validation failed: {}", e);
                    GameConfig::default()
                } else {
                    config
                }
            }
            Err(e) => {
                log::warn!("Failed to load config: {}, using defaults", e);
                let default_config = GameConfig::default();
                if let Err(save_err) = default_config.save_to_file(path) {
                    log::error!("Failed to save default config: {}", save_err);
                }
                default_config
            }
        }
    }

    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut GameConfig {
        &mut self.config
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut GameConfig),
    {
        updater(&mut self.config);
        
        if let Err(e) = self.config.validate() {
            return Err(e);
        }
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        self.config.save_to_file(&self.config_path)
    }

    pub fn reload(&mut self) -> Result<(), ConfigError> {
        self.config = GameConfig::load_from_file(&self.config_path)?;
        self.config.validate()?;
        Ok(())
    }

    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    pub fn reset_to_defaults(&mut self) -> Result<(), ConfigError> {
        self.config = GameConfig::default();
        if self.auto_save {
            self.save()?;
        }
        Ok(())
    }

    pub fn export_config(&self, path: &str) -> Result<(), ConfigError> {
        self.config.save_to_file(path)
    }

    pub fn import_config(&mut self, path: &str) -> Result<(), ConfigError> {
        let imported_config = GameConfig::load_from_file(path)?;
        imported_config.validate()?;
        self.config = imported_config;
        if self.auto_save {
            self.save()?;
        }
        Ok(())
    }
}

/// DIABOLICAL Key Code Mapping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // Special keys
    Space, Enter, Escape, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    
    // Arrow keys
    Up, Down, Left, Right,
    
    // Modifier keys
    LShift, RShift, LControl, RControl, LAlt, RAlt,
    LSuper, RSuper, LMenu, RMenu,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5,
    Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    NumpadDecimal, NumpadEnter,
    
    // Mouse buttons
    LButton, RButton, MButton, X1Button, X2Button,
    
    // Other
    CapsLock, NumLock, ScrollLock,
    Pause, PrintScreen, SysReq,
    Application, Execute, Help,
    Sleep, Wake,
    
    // Unknown
    Unknown,
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Unknown
    }
}

/// DIABOLICAL Configuration Constants
pub mod constants {
    pub const DEFAULT_CONFIG_PATH: &str = "config.json";
    pub const CONFIG_VERSION: &str = "1.0.0";
    pub const MAX_RENDER_DISTANCE: u32 = 32;
    pub const MIN_RENDER_DISTANCE: u32 = 2;
    pub const MAX_FOV: f32 = 120.0;
    pub const MIN_FOV: f32 = 30.0;
    pub const MAX_VOLUME: f32 = 1.0;
    pub const MIN_VOLUME: f32 = 0.0;
    pub const MAX_MOUSE_SENSITIVITY: f32 = 10.0;
    pub const MIN_MOUSE_SENSITIVITY: f32 = 0.1;
    pub const MAX_THREADS: usize = 64;
    pub const MIN_THREADS: usize = 1;
    pub const MAX_MEMORY_MB: u32 = 16384; // 16GB
    pub const MIN_MEMORY_MB: u32 = 256;   // 256MB
}
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::engine::{World, BlockPos, BlockType};
use crate::resources::NoiseGenerator;
use glam::Vec3;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveData {
    pub version: u32,
    pub seed: u32,
    pub player_position: Vec3,
    pub player_rotation: Vec3,
    pub player_health: f32,
    pub player_inventory: Vec<(u32, u8)>, // (block_id, count)
    pub world_chunks: HashMap<(i32, i32, i32), ChunkData>,
    pub play_time: f64, // Total play time in seconds
    pub last_saved: String, // ISO timestamp
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkData {
    pub blocks: Vec<u8>, // Compressed block data
    pub is_empty: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveSlot {
    pub name: String,
    pub world_name: String,
    pub created: String,
    pub modified: String,
    pub play_time: f64,
    pub screenshot: Option<String>, // Base64 encoded screenshot
    pub data: Option<SaveData>,
}

impl SaveSlot {
    pub fn new(slot_num: usize) -> Self {
        Self {
            name: format!("Save Slot {}", slot_num + 1),
            world_name: format!("World {}", slot_num + 1),
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
            play_time: 0.0,
            screenshot: None,
            data: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
    
    pub fn update_modified(&mut self) {
        self.modified = chrono::Utc::now().to_rfc3339();
    }
}

pub struct SaveManager {
    pub slots: Vec<SaveSlot>,
    pub current_slot: Option<usize>,
    pub auto_save_interval: f64, // in seconds
    pub last_auto_save: f64,
}

impl SaveManager {
    pub fn new() -> Self {
        let mut slots = Vec::new();
        for i in 0..5 {
            slots.push(SaveSlot::new(i));
        }
        
        Self {
            slots,
            current_slot: None,
            auto_save_interval: 300.0, // 5 minutes
            last_auto_save: 0.0,
        }
    }
    
    pub fn load_saves(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let saves_path = "saves/";
        std::fs::create_dir_all(saves_path)?;
        
        for i in 0..5 {
            let file_path = format!("saves/save_slot_{}.json", i);
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(slot) = serde_json::from_str::<SaveSlot>(&content) {
                    self.slots[i] = slot;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn save_slot(&mut self, slot_index: usize, world: &World, player: &Player, play_time: f64) -> Result<(), Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        // Compress world data
        let mut world_chunks = HashMap::new();
        for (&chunk_pos, chunk) in &world.chunks {
            let mut blocks = Vec::new();
            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        blocks.push(chunk.blocks[x][y][z] as u8);
                    }
                }
            }
            
            world_chunks.insert(chunk_pos, ChunkData {
                blocks,
                is_empty: chunk.is_empty,
            });
        }
        
        // Convert inventory
        let mut player_inventory = Vec::new();
        for (_i, slot) in player.inventory.slots.iter().enumerate() {
            if let Some(stack) = slot {
                player_inventory.push((stack.item as u32, stack.count));
            }
        }
        
        let save_data = SaveData {
            version: 1,
            seed: world.seed,
            player_position: player.position,
            player_rotation: player.rotation,
            player_health: player.health,
            player_inventory,
            world_chunks,
            play_time,
            last_saved: chrono::Utc::now().to_rfc3339(),
        };
        
        self.slots[slot_index].data = Some(save_data);
        self.slots[slot_index].update_modified();
        self.slots[slot_index].play_time = play_time;
        
        // Write to file
        let file_path = format!("saves/save_slot_{}.json", slot_index);
        let content = serde_json::to_string_pretty(&self.slots[slot_index])?;
        std::fs::write(file_path, content)?;
        
        Ok(())
    }
    
    pub fn load_slot(&mut self, slot_index: usize) -> Result<SaveData, Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        if let Some(ref data) = self.slots[slot_index].data {
            Ok(data.clone())
        } else {
            Err("Save slot is empty".into())
        }
    }
    
    pub fn delete_slot(&mut self, slot_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        self.slots[slot_index] = SaveSlot::new(slot_index);
        
        let file_path = format!("saves/save_slot_{}.json", slot_index);
        let _ = std::fs::remove_file(file_path);
        
        Ok(())
    }
    
    pub fn should_auto_save(&mut self, current_time: f64) -> bool {
        if current_time - self.last_auto_save >= self.auto_save_interval {
            self.last_auto_save = current_time;
            true
        } else {
            false
        }
    }
}
use crate::{MenuButton, MenuAction, Rect};
use crate::save_system::{SaveManager, SaveSlot};

#[derive(Clone)]
pub enum SaveMenuAction {
    Back,
    SelectSlot(usize),
    DeleteSlot(usize),
    CreateNew,
}

pub struct SaveMenu {
    pub buttons: Vec<MenuButton>,
    pub save_manager: SaveManager,
    pub selected_slot: Option<usize>,
    pub action: Option<SaveMenuAction>,
}

impl SaveMenu {
    pub fn new() -> Self {
        let mut save_manager = SaveManager::new();
        let _ = save_manager.load_saves(); // Load existing saves
        
        Self {
            buttons: Vec::new(),
            save_manager,
            selected_slot: None,
            action: None,
        }
    }
    
    pub fn update_buttons(&mut self) {
        self.buttons.clear();
        
        let w = 0.6; let h = 0.1; let g = 0.02; let start_y = 0.3;
        
        // Title
        self.buttons.push(MenuButton {
            rect: Rect { x: 0.0, y: 0.7, w: 0.4, h: 0.05 },
            text: "SELECT WORLD".to_string(),
            action: MenuAction::Resume, // Dummy action
            hovered: false,
        });
        
        // Save slots
        for i in 0..5 {
            let y = start_y - (i as f32) * (h + g);
            let slot = &self.save_manager.slots[i];
            
            let text = if slot.is_empty() {
                format!("Slot {} - [EMPTY]", i + 1)
            } else {
                format!("Slot {} - {} ({:.1}h)", i + 1, slot.world_name, slot.play_time / 3600.0)
            };
            
            self.buttons.push(MenuButton {
                rect: Rect { x: 0.0, y, w, h },
                text,
                action: MenuAction::Resume, // Will be handled specially
                hovered: false,
            });
        }
        
        // Back button
        self.buttons.push(MenuButton {
            rect: Rect { x: 0.0, y: -0.7, w: 0.3, h: 0.08 },
            text: "BACK".to_string(),
            action: MenuAction::Quit, // Use Quit as back
            hovered: false,
        });
    }
    
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<SaveMenuAction> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.rect.contains(x, y) {
                match i {
                    0 => return None, // Title, no action
                    6 => return Some(SaveMenuAction::Back), // Back button
                    slot_idx => {
                        let slot_num = slot_idx - 1; // Adjust for title button
                        if slot_num < 5 {
                            if self.save_manager.slots[slot_num].is_empty() {
                                return Some(SaveMenuAction::SelectSlot(slot_num));
                            } else {
                                // Could add confirmation dialog here
                                return Some(SaveMenuAction::SelectSlot(slot_num));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    pub fn get_slot_info(&self, slot: usize) -> Option<&SaveSlot> {
        if slot < 5 {
            Some(&self.save_manager.slots[slot])
        } else {
            None
        }
    }
}
