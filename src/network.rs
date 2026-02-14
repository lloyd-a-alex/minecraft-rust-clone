use crate::network::Packet;
use std::thread;
use std::time::{Duration, Instant};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StressTestConfig {
    pub num_clients: usize,
    pub connection_delay_ms: u64,
    pub movement_frequency_hz: f64,
    pub block_update_frequency_hz: f64,
    pub test_duration_seconds: u64,
    pub server_address: String,
    pub username_prefix: String,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            num_clients: 10,
            connection_delay_ms: 100,
            movement_frequency_hz: 5.0,
            block_update_frequency_hz: 2.0,
            test_duration_seconds: 60,
            server_address: "127.0.0.1:8080".to_string(),
            username_prefix: "StressBot".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientMetrics {
    pub client_id: usize,
    pub connected: bool,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_time: Option<Instant>,
    pub last_activity: Option<Instant>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StressTestResults {
    pub config: StressTestConfig,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub client_metrics: Vec<ClientMetrics>,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub successful_connections: usize,
    pub failed_connections: usize,
}

pub struct StressTestManager {
    config: StressTestConfig,
    results: Arc<Mutex<StressTestResults>>,
    running: Arc<Mutex<bool>>,
    client_handles: Vec<thread::JoinHandle<()>>,
}

impl StressTestManager {
    pub fn new(config: StressTestConfig) -> Self {
        let results = Arc::new(Mutex::new(StressTestResults {
            config: config.clone(),
            start_time: Instant::now(),
            end_time: None,
            client_metrics: Vec::new(),
            total_packets_sent: 0,
            total_packets_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            successful_connections: 0,
            failed_connections: 0,
        }));
        
        Self {
            config,
            results,
            running: Arc::new(Mutex::new(false)),
            client_handles: Vec::new(),
        }
    }
    
    pub fn start_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("Test is already running".into());
        }
        *running = true;
        drop(running);
        
        // Clear previous results
        {
            let mut results = self.results.lock().unwrap();
            results.start_time = Instant::now();
            results.end_time = None;
            results.client_metrics.clear();
            results.total_packets_sent = 0;
            results.total_packets_received = 0;
            results.total_bytes_sent = 0;
            results.total_bytes_received = 0;
            results.successful_connections = 0;
            results.failed_connections = 0;
        }
        
        println!("🚀 Starting stress test with {} clients", self.config.num_clients);
        
        // Spawn client threads
        for i in 0..self.config.num_clients {
            let config = self.config.clone();
            let results = Arc::clone(&self.results);
            let running = Arc::clone(&self.running);
            let delay_ms = config.connection_delay_ms;
            
            let handle = thread::spawn(move || {
                Self::run_client(i, config, results, running);
            });
            
            self.client_handles.push(handle);
            
            // Stagger connections
            if i < self.config.num_clients - 1 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        
        Ok(())
    }
    
    pub fn stop_test(&mut self) -> StressTestResults {
        let mut running = self.running.lock().unwrap();
        *running = false;
        drop(running);
        
        // Wait for all clients to finish
        for handle in self.client_handles.drain(..) {
            let _ = handle.join();
        }
        
        let mut results = self.results.lock().unwrap();
        results.end_time = Some(Instant::now());
        results.clone()
    }
    
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    
    pub fn get_results(&self) -> StressTestResults {
        self.results.lock().unwrap().clone()
    }
    
    fn run_client(
        client_id: usize,
        config: StressTestConfig,
        results: Arc<Mutex<StressTestResults>>,
        running: Arc<Mutex<bool>>,
    ) {
        let mut metrics = ClientMetrics {
            client_id,
            connected: false,
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connection_time: None,
            last_activity: None,
            errors: Vec::new(),
        };
        
        // Attempt to connect
        match std::net::TcpStream::connect(&config.server_address) {
            Ok(mut stream) => {
                metrics.connected = true;
                metrics.connection_time = Some(Instant::now());
                metrics.last_activity = Some(Instant::now());
                
                // Send handshake
                let username = format!("{}{}", config.username_prefix, client_id);
                let handshake = Packet::Handshake { 
                    username: username.clone(), 
                    seed: 12345 + client_id as u32 
                };
                
                if let Err(e) = Self::send_packet(&mut stream, &handshake) {
                    metrics.errors.push(format!("Failed to send handshake: {}", e));
                    return;
                }
                
                let start_time = Instant::now();
                let mut last_movement = start_time;
                let mut last_block_update = start_time;
                let mut movement_counter = 0;
                let mut block_counter = 0;
                
                while *running.lock().unwrap() {
                    let now = Instant::now();
                    
                    // Send movement updates
                    if now.duration_since(last_movement).as_secs_f64() >= 1.0 / config.movement_frequency_hz {
                        let move_packet = Packet::PlayerMove {
                            id: client_id as u32,
                            x: (movement_counter as f32 * 10.0).sin() * 50.0,
                            y: 64.0 + (movement_counter as f32 * 0.1).sin() * 5.0,
                            z: (movement_counter as f32 * 10.0).cos() * 50.0,
                            ry: (movement_counter as f32 * 0.05) * std::f32::consts::PI * 2.0,
                        };
                        
                        if let Err(e) = Self::send_packet(&mut stream, &move_packet) {
                            metrics.errors.push(format!("Failed to send movement: {}", e));
                            break;
                        }
                        
                        metrics.packets_sent += 1;
                        last_movement = now;
                        movement_counter += 1;
                    }
                    
                    // Send block updates
                    if now.duration_since(last_block_update).as_secs_f64() >= 1.0 / config.block_update_frequency_hz {
                        let block_packet = Packet::BlockUpdate {
                            pos: crate::world::BlockPos {
                                x: (block_counter as i32 % 100) - 50,
                                y: 64,
                                z: ((block_counter / 100) as i32 % 100) - 50,
                            },
                            block: crate::world::BlockType::Stone,
                        };
                        
                        if let Err(e) = Self::send_packet(&mut stream, &block_packet) {
                            metrics.errors.push(format!("Failed to send block update: {}", e));
                            break;
                        }
                        
                        metrics.packets_sent += 1;
                        last_block_update = now;
                        block_counter += 1;
                    }
                    
                    // Try to receive data
                    match Self::try_receive_packet(&mut stream) {
                        Ok(Some(_packet)) => {
                            metrics.packets_received += 1;
                            metrics.last_activity = Some(Instant::now());
                        }
                        Ok(None) => {
                            // No data available
                        }
                        Err(e) => {
                            metrics.errors.push(format!("Receive error: {}", e));
                            break;
                        }
                    }
                    
                    thread::sleep(Duration::from_millis(10));
                }
                
                // Send disconnect
                let _ = Self::send_packet(&mut stream, &Packet::Disconnect);
            }
            Err(e) => {
                metrics.errors.push(format!("Connection failed: {}", e));
            }
        }
        
        // Update results
        {
            let mut results = results.lock().unwrap();
            results.client_metrics.push(metrics);
            
            if results.client_metrics[client_id].connected {
                results.successful_connections += 1;
            } else {
                results.failed_connections += 1;
            }
        }
    }
    
    fn send_packet(stream: &mut std::net::TcpStream, packet: &Packet) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(packet)?;
        let len = data.len() as u32;
        
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&data)?;
        stream.flush()?;
        
        Ok(())
    }
    
    fn try_receive_packet(stream: &mut std::net::TcpStream) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
        stream.set_nonblocking(true)?;
        
        let mut len_bytes = [0u8; 4];
        match stream.read_exact(&mut len_bytes) {
            Ok(_) => {
                let len = u32::from_le_bytes(len_bytes) as usize;
                if len > 65536 { // Reasonable max packet size
                    return Err("Packet too large".into());
                }
                
                let mut data = vec![0u8; len];
                stream.read_exact(&mut data)?;
                
                let packet: Packet = bincode::deserialize(&data)?;
                Ok(Some(packet))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl StressTestResults {
    pub fn print_summary(&self) {
        let duration = self.end_time.unwrap_or(Instant::now()).duration_since(self.start_time);
        let duration_secs = duration.as_secs_f32();
        
        println!("\n📊 STRESS TEST RESULTS");
        println!("====================");
        println!("Test Duration: {:.2} seconds", duration_secs);
        println!("Target Clients: {}", self.config.num_clients);
        println!("Successful Connections: {}", self.successful_connections);
        println!("Failed Connections: {}", self.failed_connections);
        println!("Success Rate: {:.1}%", (self.successful_connections as f32 / self.config.num_clients as f32) * 100.0);
        
        if self.successful_connections > 0 {
            println!("\n📈 NETWORK STATISTICS:");
            println!("Total Packets Sent: {}", self.total_packets_sent);
            println!("Total Packets Received: {}", self.total_packets_received);
            println!("Total Bytes Sent: {} KB", self.total_bytes_sent / 1024);
            println!("Total Bytes Received: {} KB", self.total_bytes_received / 1024);
            println!("Packets/sec: {:.1}", self.total_packets_sent as f32 / duration_secs);
            println!("Throughput: {:.1} KB/s", (self.total_bytes_sent + self.total_bytes_received) as f32 / duration_secs / 1024.0);
        }
        
        if !self.client_metrics.is_empty() {
            println!("\n🔍 CLIENT DETAILS:");
            for metrics in &self.client_metrics {
                let status = if metrics.connected { "✅ Connected" } else { "❌ Failed" };
                println!("  Client {}: {} (Sent: {}, Recv: {}, Errors: {})", 
                         metrics.client_id, status, metrics.packets_sent, metrics.packets_received, metrics.errors.len());
                
                if !metrics.errors.is_empty() {
                    for error in &metrics.errors {
                        println!("    Error: {}", error);
                    }
                }
            }
        }
    }
}
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::process::Command;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os_type: OSType,
    pub architecture: String,
    pub is_supported: bool,
    pub recommended_settings: PlatformSettings,
    pub resource_urls: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OSType {
    Windows,
    MacOS,
    Linux,
    FreeBSD,
    Solaris,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    pub max_texture_size: u32,
    pub render_distance: u32,
    pub max_fps: u32,
    pub memory_limit_mb: u32,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDownloader {
    pub base_url: String,
    pub cache_dir: PathBuf,
    pub downloaded_resources: HashMap<String, ResourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub url: String,
    pub local_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
    pub version: String,
    pub last_updated: String,
}

pub struct CrossPlatformSystem {
    pub platform_info: PlatformInfo,
    pub resource_downloader: ResourceDownloader,
    pub compatibility_mode: CompatibilityMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityMode {
    Auto,
    Minimal,
    Standard,
    Maximum,
}

impl CrossPlatformSystem {
    pub fn new() -> Self {
        let platform_info = Self::detect_platform();
        let resource_downloader = ResourceDownloader::new(&platform_info);
        
        Self {
            platform_info,
            resource_downloader,
            compatibility_mode: CompatibilityMode::Auto,
        }
    }

    pub fn detect_platform() -> PlatformInfo {
        let os_type = Self::get_os_type();
        let architecture = Self::get_architecture();
        let is_supported = Self::is_platform_supported(&os_type, &architecture);
        let recommended_settings = Self::get_recommended_settings(&os_type, &architecture);
        let resource_urls = Self::get_resource_urls(&os_type);

        PlatformInfo {
            os_type,
            architecture,
            is_supported,
            recommended_settings,
            resource_urls,
        }
    }

    fn get_os_type() -> OSType {
        match env::consts::OS {
            "windows" => OSType::Windows,
            "macos" => OSType::MacOS,
            "linux" => OSType::Linux,
            "freebsd" => OSType::FreeBSD,
            "solaris" => OSType::Solaris,
            _ => OSType::Unknown,
        }
    }

    fn get_architecture() -> String {
        env::consts::ARCH.to_string()
    }

    fn is_platform_supported(os_type: &OSType, arch: &str) -> bool {
        match (os_type, arch) {
            // Windows - x86_64 and ARM64
            (OSType::Windows, "x86_64") | (OSType::Windows, "aarch64") => true,
            // macOS - Intel and Apple Silicon
            (OSType::MacOS, "x86_64") | (OSType::MacOS, "aarch64") => true,
            // Linux - Most common architectures
            (OSType::Linux, "x86_64") | (OSType::Linux, "x86") | 
            (OSType::Linux, "aarch64") | (OSType::Linux, "arm") => true,
            // FreeBSD - x86_64
            (OSType::FreeBSD, "x86_64") => true,
            // Solaris - x86_64
            (OSType::Solaris, "x86_64") => true,
            _ => false,
        }
    }

    fn get_recommended_settings(os_type: &OSType, arch: &str) -> PlatformSettings {
        let (base_memory, base_threads) = match arch {
            "x86_64" | "aarch64" => (4096, 4),
            "x86" => (2048, 2),
            "arm" => (1024, 1),
            _ => (1024, 1),
        };

        let settings = match os_type {
            OSType::Windows => PlatformSettings {
                max_texture_size: 2048,
                render_distance: 8,
                max_fps: 60,
                memory_limit_mb: base_memory,
                threads: base_threads,
            },
            OSType::MacOS => PlatformSettings {
                max_texture_size: 4096,
                render_distance: 12,
                max_fps: 60,
                memory_limit_mb: base_memory,
                threads: base_threads,
            },
            OSType::Linux => PlatformSettings {
                max_texture_size: 4096,
                render_distance: 16,
                max_fps: 144,
                memory_limit_mb: base_memory,
                threads: base_threads,
            },
            OSType::FreeBSD => PlatformSettings {
                max_texture_size: 2048,
                render_distance: 8,
                max_fps: 60,
                memory_limit_mb: base_memory,
                threads: base_threads,
            },
            OSType::Solaris => PlatformSettings {
                max_texture_size: 1024,
                render_distance: 6,
                max_fps: 30,
                memory_limit_mb: base_memory / 2,
                threads: base_threads / 2,
            },
            OSType::Unknown => PlatformSettings {
                max_texture_size: 512,
                render_distance: 4,
                max_fps: 30,
                memory_limit_mb: 512,
                threads: 1,
            },
        };

        settings
    }

    fn get_resource_urls(os_type: &OSType) -> HashMap<String, String> {
        let base_url = "https://resources.minecraft-clone.com";
        let mut urls = HashMap::new();

        match os_type {
            OSType::Windows => {
                urls.insert("windows".to_string(), format!("{}/windows", base_url));
                urls.insert("windows_x86".to_string(), format!("{}/windows/x86", base_url));
                urls.insert("windows_x64".to_string(), format!("{}/windows/x64", base_url));
                urls.insert("windows_arm64".to_string(), format!("{}/windows/arm64", base_url));
            }
            OSType::MacOS => {
                urls.insert("macos".to_string(), format!("{}/macos", base_url));
                urls.insert("macos_intel".to_string(), format!("{}/macos/intel", base_url));
                urls.insert("macos_apple".to_string(), format!("{}/macos/apple", base_url));
            }
            OSType::Linux => {
                urls.insert("linux".to_string(), format!("{}/linux", base_url));
                urls.insert("linux_x64".to_string(), format!("{}/linux/x64", base_url));
                urls.insert("linux_arm".to_string(), format!("{}/linux/arm", base_url));
                urls.insert("linux_arm64".to_string(), format!("{}/linux/arm64", base_url));
            }
            OSType::FreeBSD => {
                urls.insert("freebsd".to_string(), format!("{}/freebsd", base_url));
            }
            OSType::Solaris => {
                urls.insert("solaris".to_string(), format!("{}/solaris", base_url));
            }
            OSType::Unknown => {
                urls.insert("generic".to_string(), format!("{}/generic", base_url));
            }
        }

        urls
    }

    pub fn initialize_system(&mut self) -> Result<(), String> {
        if !self.platform_info.is_supported {
            return Err(format!("Platform {} {} is not supported", 
                format!("{:?}", self.platform_info.os_type), 
                self.platform_info.architecture));
        }

        // Create cache directory
        self.resource_downloader.create_cache_dir()?;

        // Download essential resources
        self.resource_downloader.download_essential_resources()?;

        // Apply platform-specific optimizations
        self.apply_platform_optimizations()?;

        Ok(())
    }

    pub fn apply_platform_optimizations(&self) -> Result<(), String> {
        match self.platform_info.os_type {
            OSType::Windows => {
                // Windows-specific optimizations
                self.set_windows_priority()?;
                self.configure_windows_gpu()?;
            }
            OSType::MacOS => {
                // macOS-specific optimizations
                self.set_macos_priority()?;
                self.configure_metal_renderer()?;
            }
            OSType::Linux => {
                // Linux-specific optimizations
                self.set_linux_priority()?;
                self.configure_opengl_extensions()?;
            }
            _ => {
                // Generic optimizations
                self.set_generic_optimizations()?;
            }
        }
        Ok(())
    }

    fn set_windows_priority(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
            use winapi::um::winbase::HIGH_PRIORITY_CLASS;
            
            unsafe {
                let process = GetCurrentProcess();
                if SetPriorityClass(process, HIGH_PRIORITY_CLASS) == 0 {
                    return Err("Failed to set process priority".to_string());
                }
            }
        }
        Ok(())
    }

    fn configure_windows_gpu(&self) -> Result<(), String> {
        // Configure GPU settings for Windows
        // This would interface with DirectX/OpenGL drivers
        Ok(())
    }

    fn set_macos_priority(&self) -> Result<(), String> {
        // Set process priority on macOS
        Command::new("renice")
            .arg("-20")
            .arg("-p")
            .arg(std::process::id().to_string())
            .output()
            .map_err(|_| "Failed to set process priority".to_string())?;
        Ok(())
    }

    fn configure_metal_renderer(&self) -> Result<(), String> {
        // Configure Metal renderer for macOS
        Ok(())
    }

    fn set_linux_priority(&self) -> Result<(), String> {
        // Set process priority on Linux
        Command::new("renice")
            .arg("-20")
            .arg("-p")
            .arg(std::process::id().to_string())
            .output()
            .map_err(|_| "Failed to set process priority".to_string())?;
        Ok(())
    }

    fn configure_opengl_extensions(&self) -> Result<(), String> {
        // Configure OpenGL extensions for Linux
        Ok(())
    }

    fn set_generic_optimizations(&self) -> Result<(), String> {
        // Generic optimizations for unknown platforms
        Ok(())
    }

    pub fn get_system_info(&self) -> String {
        format!(
            "Platform: {}\nArchitecture: {}\nSupported: {}\nRecommended Settings: {} texture size, {} render distance, {} FPS, {}MB memory, {} threads",
            format!("{:?}", self.platform_info.os_type),
            self.platform_info.architecture,
            self.platform_info.is_supported,
            self.platform_info.recommended_settings.max_texture_size,
            self.platform_info.recommended_settings.render_distance,
            self.platform_info.recommended_settings.max_fps,
            self.platform_info.recommended_settings.memory_limit_mb,
            self.platform_info.recommended_settings.threads
        )
    }

    pub fn set_compatibility_mode(&mut self, mode: CompatibilityMode) {
        self.compatibility_mode = mode;
        self.apply_compatibility_settings();
    }

    fn apply_compatibility_settings(&self) {
        let settings = match self.compatibility_mode {
            CompatibilityMode::Minimal => PlatformSettings {
                max_texture_size: 256,
                render_distance: 2,
                max_fps: 30,
                memory_limit_mb: 256,
                threads: 1,
            },
            CompatibilityMode::Standard => self.platform_info.recommended_settings.clone(),
            CompatibilityMode::Maximum => PlatformSettings {
                max_texture_size: 8192,
                render_distance: 32,
                max_fps: 240,
                memory_limit_mb: 8192,
                threads: num_cpus::get() as u32,
            },
            CompatibilityMode::Auto => self.platform_info.recommended_settings.clone(),
        };

        // Apply settings to the game
        // This would interface with the renderer and other systems
    }
}

impl ResourceDownloader {
    pub fn new(platform_info: &PlatformInfo) -> Self {
        let cache_dir = Self::get_cache_dir(platform_info);
        
        Self {
            base_url: "https://resources.minecraft-clone.com".to_string(),
            cache_dir,
            downloaded_resources: HashMap::new(),
        }
    }

    fn get_cache_dir(platform_info: &PlatformInfo) -> PathBuf {
        let mut cache_dir = match platform_info.os_type {
            OSType::Windows => {
                if let Some(app_data) = env::var_os("LOCALAPPDATA") {
                    PathBuf::from(app_data).join("minecraft-clone")
                } else {
                    PathBuf::from("./cache")
                }
            }
            OSType::MacOS => {
                if let Some(home) = env::var_os("HOME") {
                    PathBuf::from(home).join("Library/Application Support/minecraft-clone")
                } else {
                    PathBuf::from("./cache")
                }
            }
            _ => {
                if let Some(home) = env::var_os("HOME") {
                    PathBuf::from(home).join(".minecraft-clone")
                } else {
                    PathBuf::from("./cache")
                }
            }
        };

        fs::create_dir_all(&cache_dir).ok();
        cache_dir
    }

    pub fn create_cache_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        Ok(())
    }

    pub fn download_resource(&mut self, name: &str, url: &str) -> Result<(), String> {
        let local_path = self.cache_dir.join(name);
        
        // Check if resource already exists and is up to date
        if local_path.exists() {
            if let Ok(metadata) = fs::metadata(&local_path) {
                if let Ok(modified) = metadata.modified() {
                    let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                    if age.as_secs() < 86400 { // Less than 1 day old
                        return Ok(());
                    }
                }
            }
        }

        // Download the resource
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Failed to fetch resource: {}", e))?;

        let data = response.bytes()
            .map_err(|e| format!("Failed to read response: {}", e))?;
        let data_len = data.len();

        fs::write(&local_path, &data)
                .map_err(|e| format!("Failed to write resource: {}", e))?;

        // Update resource info
        self.downloaded_resources.insert(name.to_string(), ResourceInfo {
            url: url.to_string(),
            local_path: local_path.clone(),
            size_bytes: data_len as u64,
            checksum: format!("{:x}", md5::compute(&data)),
            version: "1.0".to_string(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        });

        Ok(())
    }

    pub fn download_essential_resources(&mut self) -> Result<(), String> {
        let essential_resources = vec![
            ("textures/basic.png", "https://resources.minecraft-clone.com/textures/basic.png"),
            ("shaders/basic.wgsl", "https://resources.minecraft-clone.com/shaders/basic.wgsl"),
            ("sounds/basic.ogg", "https://resources.minecraft-clone.com/sounds/basic.ogg"),
        ];

        for (name, url) in essential_resources {
            self.download_resource(name, url)?;
        }

        Ok(())
    }

    pub fn get_resource_path(&self, name: &str) -> Option<PathBuf> {
        self.downloaded_resources.get(name).map(|info| info.local_path.clone())
    }

    pub fn cleanup_old_resources(&mut self) -> Result<(), String> {
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(7 * 24 * 60 * 60); // 7 days

        for (name, info) in &self.downloaded_resources.clone() {
            if let Ok(modified) = fs::metadata(&info.local_path).and_then(|m| m.modified()) {
                if modified < cutoff {
                    fs::remove_file(&info.local_path).ok();
                    self.downloaded_resources.remove(name);
                }
            }
        }

        Ok(())
    }
}
use std::io::{Read, Write};
use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::configuration::GameConfig;
use crate::engine::{World, BlockPos, BlockType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Packet {
    Handshake { username: String, seed: u32 },
    PlayerMove { id: u32, x: f32, y: f32, z: f32, ry: f32 },
    BlockUpdate { pos: BlockPos, block: BlockType },
    Disconnect,
}

#[allow(dead_code)]
pub struct NetworkManager {
    pub is_server: bool,
    pub stream: Option<TcpStream>,
    sender: Sender<Packet>,
    receiver: Receiver<Packet>,
    pub my_id: u32,
    pub seed: Option<u32>,
}

impl Packet {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Packet::Handshake { username, seed } => {
                if username.len() > 32 {
                    return Err("Username too long (max 32 characters)".to_string());
                }
                if username.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
                    return Err("Username contains invalid characters".to_string());
                }
                // Validate seed is within reasonable range
                if *seed > u32::MAX / 2 {
                    return Err("Invalid seed value".to_string());
                }
            }
            Packet::PlayerMove { id, x, y, z, ry } => {
                if *id > 10000 {
                    return Err("Invalid player ID".to_string());
                }
                // Validate coordinates are within reasonable world bounds
                if !x.is_finite() || !y.is_finite() || !z.is_finite() || !ry.is_finite() {
                    return Err("Invalid coordinates (NaN or infinite)".to_string());
                }
                if x.abs() > 100000.0 || y.abs() > 100000.0 || z.abs() > 100000.0 {
                    return Err("Coordinates out of bounds".to_string());
                }
            }
            Packet::BlockUpdate { pos, block } => {
                // Validate block position
                if pos.x.abs() > 10000 || pos.y.abs() > 1000 || pos.z.abs() > 10000 {
                    return Err("Block position out of bounds".to_string());
                }
                // Validate block type is within enum range
                if (*block as u8) > 200 {
                    return Err("Invalid block type".to_string());
                }
            }
            Packet::Disconnect => {
                // Disconnect packet is always valid
            }
        }
        Ok(())
    }
}

impl NetworkManager {
    pub fn host(_port: String, seed: u32) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out) = unbounded();

        // DIABOLICAL FIX: 0.0.0.0 binds to EVERY interface (LAN, Hamachi, Ngrok) simultaneously
        let address = "0.0.0.0:25565";
        println!("🔥 HOSTING SERVER ON: {}", address);

        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                log::error!("Failed to bind to port {}: {:?}", address, e);
                panic!("Failed to bind to port");
            }
        };
        if let Err(e) = listener.set_nonblocking(true) {
            log::error!("Failed to set non-blocking mode: {:?}", e);
        }

        let tx_in_clone = tx_in.clone();

        thread::spawn(move || {
            let mut client_id_counter = 2; // Host is 1
            loop {
                if let Ok((mut stream, addr)) = listener.accept() {
                    println!("✨ NEW PLAYER CONNECTED: {:?} (ID: {})", addr, client_id_counter);

                    let _ = stream.set_nonblocking(false);

                    // --- RADICAL MULTIPLAYER HANDSHAKE ---
                    // Forcefully sync the seed and ensure the client receives it before spawning
                    let handshake = Packet::Handshake { username: "Host".to_string(), seed };
                    if let Ok(bytes) = bincode::serialize(&handshake) {
                        let _ = stream.write_all(&bytes);
                        let _ = stream.flush();
                    }
                    // --------------------------------------

                    let mut stream_clone = match stream.try_clone() {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("Failed to clone stream: {:?}", e);
                            continue;
                        }
                    };
                    let tx_in_thread = tx_in_clone.clone();

                    // Reader
                    thread::spawn(move || {
                        let mut buffer = [0u8; 1024];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Ok(packet) = bincode::deserialize::<Packet>(&buffer[..n]) {
                                        // Validate packet before processing
                                        if let Err(e) = packet.validate() {
                                            log::warn!("Invalid packet received: {}", e);
                                            continue;
                                        }
                                        if let Err(e) = tx_in_thread.send(packet) {
                                            log::debug!("Failed to send packet to main thread: {:?}", e);
                                            break;
                                        }
                                    } else {
                                        log::warn!("Failed to deserialize packet of {} bytes", n);
                                    }
                                }
                                Err(_) => {}
                            }
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                    });

                    // Writer (Broadcaster)
                    let rx_out_thread: crossbeam_channel::Receiver<Packet> = rx_out.clone();
                    thread::spawn(move || {
                        while let Ok(packet) = rx_out_thread.recv() {
                            // Validate packet before sending
                            if let Err(e) = packet.validate() {
                                log::warn!("Attempted to send invalid packet: {}", e);
                                continue;
                            }
                            match bincode::serialize(&packet) {
                                Ok(encoded) => {
                                    if stream_clone.write_all(&encoded).is_err() {
                                        log::debug!("Failed to write packet to stream");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to serialize packet: {:?}", e);
                                }
                            }
                        }
                    });

                    client_id_counter += 1;
                }
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });

NetworkManager {
            is_server: true,
            stream: None,
            sender: tx_out,
            receiver: rx_in,
            my_id: 1,
            seed: Some(seed),
        }
    }

    pub fn join(mut ip: String) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out): (crossbeam_channel::Sender<Packet>, crossbeam_channel::Receiver<Packet>) = unbounded();

        // Sanitize Ngrok/SSH Tunnel addresses
        if ip.starts_with("tcp://") { ip = ip.replace("tcp://", ""); }
        if ip.starts_with("http://") { ip = ip.replace("http://", ""); }

        println!("🚀 CONNECTING TO: {}", ip);
        
        // --- RETRY LOGIC (60 Seconds Timeout) ---
        let start = std::time::Instant::now();
        let stream = loop {
            match TcpStream::connect(&ip) {
                Ok(s) => break s,
                Err(_) => {
                    if start.elapsed().as_secs() > 60 { // Increased to 60s
                        panic!("❌ CONNECTION TIMED OUT: Could not find server at {}", ip);
                    }
                    print!("."); 
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        };
        println!("\n✅ DIABOLICAL CONNECTION ESTABLISHED!");

        let mut stream_read = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to clone stream for reading: {:?}", e);
                panic!("Failed to clone stream");
            }
        };
        let mut stream_write = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to clone stream for writing: {:?}", e);
                panic!("Failed to clone stream");
            }
        };

        // Reader
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match stream_read.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(packet) = bincode::deserialize::<Packet>(&buffer[..n]) {
                            // Validate packet before processing
                            if let Err(e) = packet.validate() {
                                log::warn!("Invalid packet received from server: {}", e);
                                continue;
                            }
                            // DIABOLICAL FIX: Handle channel disconnect gracefully (world rebuild/quit)
                            if tx_in.send(packet).is_err() { break; } 
                        } else {
                            log::warn!("Failed to deserialize packet of {} bytes from server", n);
                        }
                    }
                    Err(_) => {}
                }
                thread::sleep(std::time::Duration::from_millis(5));
            }
        });

        // Writer
        thread::spawn(move || {
            while let Ok(packet) = rx_out.recv() {
                // Validate packet before sending
                if let Err(e) = packet.validate() {
                    log::warn!("Attempted to send invalid packet to server: {}", e);
                    continue;
                }
                match bincode::serialize(&packet) {
                    Ok(encoded) => {
                        if stream_write.write_all(&encoded).is_err() { 
                            log::debug!("Failed to write packet to server stream");
                            break; 
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to serialize packet for server: {:?}", e);
                    }
                }
            }
        });

NetworkManager {
            is_server: false,
            stream: Some(stream),
            sender: tx_out,
            receiver: rx_in,
            my_id: 2,
            seed: None,
        }
    }

    pub fn send_packet(&self, packet: Packet) {
        let _ = self.sender.send(packet);
    }

    pub fn try_recv(&self) -> Option<Packet> {
        self.receiver.try_recv().ok()
    }
}
use std::process::{Command, Child, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};
use std::fs;
use std::path::Path;
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DiscoveredServer {
    pub name: String,
    pub address: String,
    pub last_seen: Instant,
}

pub struct HostingManager {
    pub ngrok_process: Option<Child>,
    pub ssh_process: Option<Child>,
    pub public_url: Arc<Mutex<String>>, // ARC for safe in-game UI access
    pub hamachi_ip: Option<String>,
    pub wan_ip: Option<String>,
    pub lan_ip: Option<String>,
    pub discovered_servers: Arc<Mutex<Vec<DiscoveredServer>>>,
}

impl HostingManager {
    pub fn new() -> Self {
        // DIABOLICAL CLEANUP: Forcefully kill any zombie ngrok processes before starting
        #[cfg(target_os = "windows")]
        let _ = Command::new("taskkill").args(&["/F", "/IM", "ngrok.exe", "/T"]).output();
        #[cfg(not(target_os = "windows"))]
        let _ = Command::new("pkill").arg("-9").arg("ngrok").output();

        let manager = Self {
            ngrok_process: None,
            ssh_process: None,
            public_url: Arc::new(Mutex::new("Initializing...".to_string())),
            hamachi_ip: None,
            wan_ip: None,
            lan_ip: None,
            discovered_servers: Arc::new(Mutex::new(Vec::new())),
        };
        manager.start_discovery_listener();
        manager
    }

    pub fn init(&mut self) {
        log::info!("╔════════════════════════════════════════════════════════════╗");
        log::info!("║ 🌐 INITIALIZING HYPER-HOSTING MULTIPLAYER PROTOCOL...      ║");
        log::info!("╚════════════════════════════════════════════════════════════╝");

        // 1. Detect LAN IP (Primary for same-house play)
        self.lan_ip = self.detect_lan_ip();

        // 2. DIABOLICAL CLEANUP: Kill any lingering ngrok processes to prevent ERR_NGROK_108
        if cfg!(target_os = "windows") {
            let _ = Command::new("taskkill").args(&["/F", "/IM", "ngrok.exe", "/T"]).output();
        } else {
            let _ = Command::new("pkill").arg("-9").arg("ngrok").output();
        }
        thread::sleep(Duration::from_millis(500));
        
        // 1. Detect VPN/Hamachi Adapters
        self.hamachi_ip = self.detect_hamachi();
        if let Some(ref ip) = self.hamachi_ip {
            log::info!("🛡️  HAMACHI/VPN DETECTED: {}", ip);
        }

        // 2. Auto-Detect Public WAN IP (Manual Port Forward Fallback)
        self.wan_ip = self.fetch_public_ip();

        // 3. Setup Ngrok (Professional TCP Tunneling)
        self.setup_ngrok();

        // 4. Setup SSH Fallback (localhost.run)
        let url_val = self.public_url.lock().unwrap().clone();
        if url_val == "Initializing..." || url_val.is_empty() {
            self.setup_ssh_tunnel();
        }

        // 5. Start LAN Discovery Beacon
        self.start_lan_beacon();
        
        log::info!("✅ HYPER-HOSTING ACTIVE. MULTI-CHANNEL ADVERTISING:");
        let final_url = self.public_url.lock().unwrap().clone();
        if !final_url.is_empty() { log::info!("   - TUNNEL:  {}", final_url); }
        if let Some(ref ip) = self.wan_ip { log::info!("   - WAN IP:  {} (Req. Port Forward 25565)", ip); }
        if let Some(ref h) = self.hamachi_ip { log::info!("   - VPN:     {}", h); }
        
        self.print_dashboard();
    }

    fn print_dashboard(&self) {
        let lan_ip = self.lan_ip.clone().unwrap_or_else(|| "127.0.0.1".to_string());
        let wan_url = self.public_url.lock().unwrap().clone();

        log::info!("╔════════════════════════════════════════════════════════════╗");
        log::info!("║            🌍 MULTIPLAYER JOIN DASHBOARD 🌍                ║");
        log::info!("╠════════════════════════════════════════════════════════════╣");
        log::info!("║  1. JOIN VIA LAN (Same House):                             ║");
        log::info!("║     IP: {:<46} ║", format!("{}:25565", lan_ip));
        log::info!("║                                                            ║");
        log::info!("║  2. JOIN VIA WAN (Friends Anywhere):                       ║");
        if wan_url == "Initializing..." || wan_url.is_empty() {
            log::info!("║     IP: [TUNNEL STARTING... RE-CHECKING SOON]              ║");
        } else {
            log::info!("║     IP: {:<46} ║", wan_url);
        }
        log::info!("╠════════════════════════════════════════════════════════════╣");
        log::info!("║  INSTRUCTIONS FOR FRIENDS:                                 ║");
        log::info!("║  - Copy the WAN IP above (including the port).             ║");
        log::info!("║  - Go to 'Connect to Server' in the game menu.             ║");
        log::info!("║  - Paste and Hit Connect. Stay diabolical!                 ║");
        log::info!("╚════════════════════════════════════════════════════════════╝");
    }

    fn detect_lan_ip(&self) -> Option<String> {
        if cfg!(target_os = "windows") {
            let output = Command::new("powershell")
                .args(&["-Command", "(Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.InterfaceAlias -notlike '*Loopback*' -and $_.InterfaceAlias -notlike '*Virtual*' -and $_.IPv4Address -notlike '169.254.*'}).IPv4Address | Select-Object -First 1"])
                .output().ok()?;
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let output = Command::new("sh").arg("-c").arg("hostname -I | awk '{print $1}'").output().ok()?;
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
    }

    fn fetch_public_ip(&self) -> Option<String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell").args(&["-Command", "Invoke-RestMethod -Uri 'https://api.ipify.org'"]).output().ok()
        } else {
            Command::new("curl").arg("-s").arg("https://api.ipify.org").output().ok()
        };
        output.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    fn setup_ssh_tunnel(&mut self) {
        log::info!("🔑 STARTING SSH FALLBACK (localhost.run)...");
        let child = Command::new("ssh")
            .args(&["-o", "StrictHostKeyChecking=no", "-R", "80:localhost:25565", "nokey@localhost.run"])
            .stdout(Stdio::piped())
            .spawn();

        if let Ok(mut c) = child {
            if let Some(stdout) = c.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    if line.contains(".lhr.life") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for p in parts { 
                            if p.contains(".lhr.life") { 
                                *self.public_url.lock().unwrap() = p.to_string(); 
                                break; 
                            } 
                        }
                        log::info!("🚀 SSH TUNNEL ACTIVE: {}", self.public_url.lock().unwrap());
                        break;
                    }
                }
            }
            self.ssh_process = Some(c);
        }
    }

    fn detect_hamachi(&self) -> Option<String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("ipconfig").output().ok()?
        } else {
            Command::new("ifconfig").output().ok()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut in_hamachi_section = false;
        
        for line in stdout.lines() {
            if line.contains("Hamachi") || line.contains("ZeroTier") { in_hamachi_section = true; }
            if in_hamachi_section && (line.contains("IPv4 Address") || line.contains("inet ")) {
                return line.split(':').last()?.split_whitespace().next()?.trim().to_string().into();
            }
        }
        None
    }

    fn setup_ngrok(&mut self) {
        let ngrok_bin = if cfg!(target_os = "windows") { "ngrok.exe" } else { "ngrok" };
        let ngrok_path = format!("./{}", ngrok_bin);
        
        // Ensure log indentation isn't ruined by child process output
        let _ = std::io::stdout().flush();

        if !Path::new(&ngrok_path).exists() {
            log::warn!("⚠️  NGROK BINARY MISSING. ATTEMPTING AUTOMATIC CROSS-PLATFORM FETCH...");
            if let Err(e) = self.download_ngrok_cross_platform() {
                log::error!("❌ AUTO-DOWNLOAD FAILED: {}. Falling back to LAN.", e);
                return;
            }
        }

        // Apply Auth Token if present in root
        if let Ok(token) = fs::read_to_string("ngrok_token.txt") {
            let _ = Command::new(&ngrok_path).args(&["config", "add-authtoken", token.trim()]).output();
        }

        // Start Tunnel on the "Golden Port" 25565
        let child = Command::new(&ngrok_path)
            .args(&["tcp", "25565"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if let Ok(c) = child {
            self.ngrok_process = Some(c);
            
            // DIABOLICAL API POLLING: Don't trust stdout, ask the ngrok daemon directly
            let url_clone = Arc::clone(&self.public_url);
            thread::spawn(move || {
                for _ in 0..20 {
                    thread::sleep(Duration::from_millis(1000));
                    let output = if cfg!(target_os = "windows") {
                        Command::new("powershell").args(&["-Command", "Invoke-RestMethod -Uri 'http://127.0.0.1:4040/api/tunnels' | ConvertTo-Json"]).output()
                    } else {
                        Command::new("curl").args(&["-s", "http://127.0.0.1:4040/api/tunnels"]).output()
                    };

                    if let Ok(out) = output {
                        let body = String::from_utf8_lossy(&out.stdout);
                        if let Some(pos) = body.find("tcp://") {
                            let end = body[pos..].find("\"").unwrap_or(30);
                            let found = body[pos..pos+end].replace("\\u0026", "&");
                            *url_clone.lock().unwrap() = found.clone();
                            log::info!("🚀 GLOBAL TUNNEL ACTIVE: {}", found);
                            return;
                        }
                    }
                }
                log::error!("❌ NGROK API TIMEOUT: Tunnel may have failed to start.");
            });
        }
    }

    fn download_ngrok_cross_platform(&self) -> Result<(), Box<dyn std::error::Error>> {
        let target = match (env::consts::OS, env::consts::ARCH) {
            ("windows", "x86")    => "windows-386",
            ("windows", _)        => "windows-amd64",
            ("macos", "aarch64")  => "darwin-arm64",
            ("macos", _)          => "darwin-amd64",
            ("linux", "aarch64")  => "linux-arm64",
            ("linux", "arm")      => "linux-arm",
            ("linux", _)          => "linux-amd64",
            ("freebsd", "x86")    => "freebsd-386",
            ("freebsd", _)        => "freebsd-amd64",
            _ => return Err("Unsupported OS Architecture".into()), 
        };

        log::info!("⬇️  FETCHING NGROK FOR PLATFORM: {}", target);
        let url = format!("https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip", target);
        
        // Use PowerShell for Windows Zero-Setup bypass
        if cfg!(target_os = "windows") {
            let cmd = format!(
                "Invoke-WebRequest -Uri '{}' -OutFile 'ngrok.zip'; Expand-Archive -Path 'ngrok.zip' -DestinationPath '.'; Remove-Item 'ngrok.zip'",
                url
            );
            let status = Command::new("powershell").args(&["-Command", &cmd]).status()?;
            if !status.success() { return Err("PowerShell download failed".into()); }
        } else {
            // Unix-like systems: use curl and unzip
            let _ = Command::new("curl").args(&["-L", "-o", "ngrok.zip", &url]).status()?;
            let _ = Command::new("unzip").arg("ngrok.zip").status()?;
            let _ = Command::new("rm").arg("ngrok.zip").status()?;
            let _ = Command::new("chmod").args(&["+x", "ngrok"]).status()?;
        }
        Ok(())
    }

    fn start_lan_beacon(&self) {
        thread::spawn(|| {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Beacon bind fail");
            socket.set_broadcast(true).ok();
            let msg = "MC_RUST_CLONE_SERVER:25565";
            loop {
                // Broadcast on port 25566 so clients can listen without colliding with the game port
                let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:25566");
                thread::sleep(Duration::from_secs(5));
            }
        });
        log::info!("📡 LAN DISCOVERY BEACON ACTIVE (UDP 25566)");
    }

    fn start_discovery_listener(&self) {
        let registry = Arc::clone(&self.discovered_servers);
        thread::spawn(move || {
            let socket = UdpSocket::bind("0.0.0.0:25566").expect("Discovery Listener bind fail");
            socket.set_read_timeout(Some(Duration::from_secs(1))).ok();
            let mut buf = [0u8; 1024];

            loop {
                if let Ok((size, src)) = socket.recv_from(&mut buf) {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    if msg.starts_with("MC_RUST_CLONE_SERVER:") {
                        let parts: Vec<&str> = msg.split(':').collect();
                        let port = parts.get(1).unwrap_or(&"25565");
                        let addr = format!("{}:{}", src.ip(), port);
                        
                        let mut servers = registry.lock().unwrap();
                        if let Some(existing) = servers.iter_mut().find(|s| s.address == addr) {
                            existing.last_seen = Instant::now();
                        } else {
                            servers.push(DiscoveredServer {
                                name: format!("Local Server ({})", src.ip()),
                                address: addr,
                                last_seen: Instant::now(),
                            });
                        }
                    }
                }
                
                // Cleanup stale servers (not seen for 15s)
                let mut servers = registry.lock().unwrap();
                servers.retain(|s| s.last_seen.elapsed() < Duration::from_secs(15));
                drop(servers);
                thread::sleep(Duration::from_millis(100));
            }
        });
        println!("\n\n"); // DIABOLICAL FORMATTING FIX: Clear the console buffer after Ngrok/SSH spam
        log::info!("👂 DISCOVERY LISTENER ACTIVE");
    }
}
