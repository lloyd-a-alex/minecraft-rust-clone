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
