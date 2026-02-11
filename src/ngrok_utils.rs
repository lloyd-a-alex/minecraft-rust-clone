use std::process::{Command, Child, Stdio};
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::fs;
use std::path::Path;

pub struct HostingManager {
    pub ngrok_process: Option<Child>,
    pub public_url: String,
    pub hamachi_ip: Option<String>,
}

impl HostingManager {
    pub fn new() -> Self {
        Self {
            ngrok_process: None,
            public_url: "Local Only".to_string(),
            hamachi_ip: None,
        }
    }

    pub fn init(&mut self) {
        log::info!("🌐 INITIALIZING HYPER-HOSTING PROTOCOL...");
        
        // 1. Detect Hamachi
        self.hamachi_ip = self.detect_hamachi();
        if let Some(ref ip) = self.hamachi_ip {
            log::info!("🛡️  HAMACHI DETECTED: {}", ip);
        }

        // 2. Setup Ngrok (Zero-Setup Downloader)
        self.setup_ngrok();

        // 3. Start LAN Discovery Beacon
        self.start_lan_beacon();
    }

    fn detect_hamachi(&self) -> Option<String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("ipconfig").output().ok()?
        } else {
            Command::new("ifconfig").output().ok()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut found_hamachi = false;
        
        for line in stdout.lines() {
            if line.contains("Hamachi") { found_hamachi = true; }
            if found_hamachi && (line.contains("IPv4 Address") || line.contains("inet ")) {
                return line.split(':').last()?.trim().to_string().into();
            }
        }
        None
    }

    fn setup_ngrok(&mut self) {
        let ngrok_path = if cfg!(target_os = "windows") { "./ngrok.exe" } else { "./ngrok" };
        
        if !Path::new(ngrok_path).exists() {
            log::warn!("⚠️  NGROK MISSING. ATTEMPTING DIABOLICAL AUTO-DOWNLOAD...");
            if cfg!(target_os = "windows") {
                let download_cmd = "Invoke-WebRequest -Uri 'https://bin.equinox.io/c/bPR9thYdyFv/ngrok-v3-stable-windows-amd64.zip' -OutFile 'ngrok.zip'; Expand-Archive -Path 'ngrok.zip' -DestinationPath '.'; Remove-Item 'ngrok.zip'";
                let _ = Command::new("powershell").args(&["-Command", download_cmd]).status();
            }
        }

        // Apply Auth Token if present
        if let Ok(token) = fs::read_to_string("ngrok_token.txt") {
            let _ = Command::new(ngrok_path).args(&["config", "add-authtoken", token.trim()]).output();
        }

        // Start Ngrok Tunnel on Minecraft Port 25565
        let child = Command::new(ngrok_path)
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
                            log::info!("🚀 NGROK TUNNEL ACTIVE: {}", self.public_url);
                            break;
                        }
                    }
                }
            }
            self.ngrok_process = Some(c);
        }
    }

    fn start_lan_beacon(&self) {
        thread::spawn(|| {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Beacon bind fail");
            socket.set_broadcast(true).ok();
            let msg = "MC_RUST_CLONE_SERVER:25565";
            loop {
                let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:25566");
                thread::sleep(Duration::from_secs(5));
            }
        });
        log::info!("📡 LAN DISCOVERY BEACON ACTIVE (Port 25566)");
    }
}