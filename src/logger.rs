use log::{info, warn, error};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .format(|buf, record| {
            // Fix: Use SystemTime to get a raw u64 number for arithmetic
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let time_str = format!("{:02}:{:02}:{:02}", (ts / 3600) % 24, (ts / 60) % 60, ts % 60);
            writeln!(buf, "[{} {:<5}] {}", time_str, record.level(), record.args())
        })
        .init();

    log::info!("╔════════════════════════════════════════════════════════════╗");
    log::info!("║ 🎮 MINECRAFT RUST CLONE - SYSTEM INITIALIZED 🎮           ║");
    log::info!("╚════════════════════════════════════════════════════════════╝");
    log::info!("Version: 1.0 | Build: CREATIVE | Log Level: DEBUG");
}

#[allow(dead_code)]
pub fn log_world_generation(chunk_count: usize, block_count: usize) {
    info!("✅ WORLD GENERATION COMPLETE");
    info!(" └─ Chunks generated: {}", chunk_count);
    info!(" └─ Total blocks: {}", block_count);
    info!(" └─ Memory: ~{} MB", (block_count * 32) / 1_000_000);
}

#[allow(dead_code)]
pub fn log_renderer_init(width: u32, height: u32) {
    info!("✅ RENDERER INITIALIZED");
    info!(" └─ Resolution: {}x{}", width, height);
    info!(" └─ Pipeline: wgpu 0.19");
    info!(" └─ Texture Atlas: 256x256 (Procedural)");
}

#[allow(dead_code)]
pub fn log_player_update(x: f32, y: f32, z: f32, block_under: &str) {
    info!("📍 PLAYER POSITION: ({:.1}, {:.1}, {:.1}) | Standing on: {}", x, y, z, block_under);
}

#[allow(dead_code)]
pub fn log_hotbar_selection(slot: usize, block: &str) {
    info!("🎯 HOTBAR SELECTED: Slot {} → {}", slot, block);
}

#[allow(dead_code)]
pub fn log_warning(msg: &str) {
    warn!("⚠️ WARNING: {}", msg);
}

#[allow(dead_code)]
pub fn log_error(msg: &str) {
    error!("❌ ERROR: {}", msg);
}