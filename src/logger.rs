use log::{info, warn, error};

pub fn init_logger() {
    // We configure the logger to be "Info" for our game, but "Error" only for the engine.
    // This suppresses the "Debug utils not enabled" and "Validation layer missing" spam.
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // Show our game logs
        .filter_module("wgpu_core", log::LevelFilter::Error) // Hide wgpu warnings
        .filter_module("wgpu_hal", log::LevelFilter::Error)  // Hide Vulkan/DX12 warnings
        .filter_module("naga", log::LevelFilter::Error)      // Hide Shader warnings
        .format_timestamp_millis()
        .init();

    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║ 🎮 MINECRAFT RUST CLONE - SYSTEM INITIALIZED 🎮           ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("Version: 1.0 | Build: CREATIVE");
    info!("Logging Level: INFO (Engine Warnings Suppressed)");
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
