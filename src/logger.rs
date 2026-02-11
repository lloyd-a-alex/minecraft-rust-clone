#[allow(unused_imports)]
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
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            let ts = now.as_secs();
            let ms = now.subsec_millis();
            let time_str = format!("{:02}:{:02}:{:02}.{:03}", (ts / 3600) % 24, (ts / 60) % 60, ts % 60, ms);
            writeln!(buf, "[{} {:<5}] {}", time_str, record.level(), record.args())
        })
        .init();

    log::info!("╔════════════════════════════════════════════════════════════╗");
    log::info!("║ 🎮 MINECRAFT RUST CLONE - SYSTEM INITIALIZED 🎮           ║");
    log::info!("╚════════════════════════════════════════════════════════════╝");
}