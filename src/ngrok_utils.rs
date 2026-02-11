use std::process::{Command, Child, Stdio};
use std::io::{BufRead, BufReader};
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
    pub public_url: String,
    pub hamachi_ip: Option<String>,
    pub wan_ip: Option<String>,
    pub discovered_servers: Arc<Mutex<Vec<DiscoveredServer>>>,
}

impl HostingManager {
    pub fn new() -> Self {
        let manager = Self {
            ngrok_process: None,
            ssh_process: None,
            public_url: "Initializing...".to_string(),
            hamachi_ip: None,
            wan_ip: None,
            discovered_servers: Arc::new(Mutex::new(Vec::new())),
        };
        manager.start_discovery_listener();
        manager
    }

    pub fn init(&mut self) {
        log::info!("╔════════════════════════════════════════════════════════════╗");
        log::info!("║ 🌐 INITIALIZING HYPER-HOSTING MULTIPLAYER PROTOCOL...      ║");
        log::info!("╚════════════════════════════════════════════════════════════╝");
        
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
        if self.public_url == "Initializing..." || self.public_url.is_empty() {
            self.setup_ssh_tunnel();
        }

        // 5. Start LAN Discovery Beacon
        self.start_lan_beacon();
        
        log::info!("✅ HYPER-HOSTING ACTIVE. MULTI-CHANNEL ADVERTISING:");
        if !self.public_url.is_empty() { log::info!("   - TUNNEL:  {}", self.public_url); }
        if let Some(ref ip) = self.wan_ip { log::info!("   - WAN IP:  {} (Req. Port Forward 25565)", ip); }
        if let Some(ref h) = self.hamachi_ip { log::info!("   - VPN:     {}", h); }
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
                        for p in parts { if p.contains(".lhr.life") { self.public_url = p.to_string(); break; } }
                        log::info!("🚀 SSH TUNNEL ACTIVE: {}", self.public_url);
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
            .args(&["tcp", "25565", "--log=stdout"])
            .stdout(Stdio::piped())
            .spawn();

        if let Ok(mut c) = child {
            if let Some(stdout) = c.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    if line.contains("url=tcp://") {
                        if let Some(pos) = line.find("url=tcp://") {
                            self.public_url = line[pos + 4..].split_whitespace().next().unwrap_or("Error").to_string();
                            log::info!("🚀 GLOBAL TUNNEL ACTIVE: {}", self.public_url);
                            break;
                        }
                    }
                }
            }
            self.ngrok_process = Some(c);
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
        log::info!("👂 DISCOVERY LISTENER ACTIVE");
    }
}