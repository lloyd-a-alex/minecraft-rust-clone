use std::fs::{self, File};
use std::io::{self, Cursor, Write, BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::env;

#[cfg(target_os = "windows")] const NGROK_BIN: &str = "ngrok.exe";
#[cfg(not(target_os = "windows"))] const NGROK_BIN: &str = "ngrok";

pub fn start_ngrok_tunnel(port: &str) -> Option<String> {
    let os = env::consts::OS.to_uppercase();
    let arch = env::consts::ARCH.to_uppercase();
    println!("----------------------------------------------------------");
    println!("🔌 NETWORK INIT | OS: {} | ARCH: {}", os, arch);

    // 1. CLEANUP (Kill old processes)
    cleanup_processes();

    // 2. NGROK ATTEMPT (10s Timeout)
    println!("🚀 Attempting Ngrok Tunnel (Method A)...");
    if let Some(url) = attempt_ngrok(port) {
        return Some(url);
    }

    // 3. FALLBACK: SSH TUNNEL (Method B - Universal)
    println!("⚠️  Ngrok failed/skipped. Attempting SSH Tunnel (Method B)...");
    println!("ℹ️  (This works on FreeBSD, Docker, and Raspberry Pi!)");
    attempt_ssh_tunnel(port)
}

fn attempt_ngrok(port: &str) -> Option<String> {
    // Check/Auth Token
    let token_file = Path::new("ngrok_token.txt");
    if token_file.exists() {
        if let Ok(token) = fs::read_to_string(token_file) {
            configure_ngrok(token.trim());
        }
    }

    // Download if missing
    let path = Path::new(NGROK_BIN);
    if !path.exists() {
        println!("⬇️  Downloading optimized Ngrok binary...");
        if let Err(_) = download_ngrok() {
            println!("❌ Ngrok download failed. Skipping to SSH...");
            return None; 
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
    }

    // Launch Process
    let log_file = File::create("ngrok.log").ok()?;
    let exe = if cfg!(target_os = "windows") { "ngrok.exe" } else { "./ngrok" };
    
    let mut child = Command::new(exe)
        .arg("tcp")
        .arg(port)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    println!("⏳ Waiting for Ngrok...");
    
    // Poll for Success (10s timeout to allow skipping)
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        
        // Did it crash? (Likely auth missing)
        if let Ok(Some(_)) = child.try_wait() {
            // Only ask for auth if we are in an interactive mode, otherwise skip
            println!("❌ Ngrok process died. Skipping...");
            return None;
        }

        // Check API
        if let Ok(resp) = reqwest::blocking::get("http://127.0.0.1:4040/api/tunnels") {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(url) = json["tunnels"][0]["public_url"].as_str() {
                    let clean = url.replace("tcp://", "");
                    println!("✅ NGROK CONNECTED: {}", clean);
                    return Some(clean.to_string());
                }
            }
        }
    }
    
    println!("⏩ Ngrok timed out. Killing process...");
    let _ = child.kill();
    None
}

fn attempt_ssh_tunnel(port: &str) -> Option<String> {
    println!("🔄 Connecting to 'localhost.run'...");
    
    // Command: ssh -R 80:localhost:PORT -o StrictHostKeyChecking=no nokey@localhost.run
    // We use port 80 mappings because they are often more stable on the free tier
    let mut child = Command::new("ssh")
        .arg("-R")
        .arg(format!("80:localhost:{}", port))
        .arg("-o").arg("StrictHostKeyChecking=no") // Don't ask for fingerprint
        .arg("nokey@localhost.run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) 
        .spawn()
        .ok()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    // Scan output for 10 seconds to find the URL
    let start = std::time::Instant::now();
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains(".localhost.run") {
                if let Some(start_idx) = l.find("https://") {
                    let url = l[start_idx..].trim().to_string();
                    let clean = url.replace("https://", ""); 
                    println!("✅ SSH TUNNEL CONNECTED: {}", clean);
                    println!("📝 NOTE: This is a Domain Address. It works just like an IP!");
                    return Some(clean);
                }
            }
        }
        if start.elapsed().as_secs() > 10 { break; }
    }
    
    println!("❌ SSH Fallback failed. Playing LAN Only.");
    let _ = child.kill();
    None
}

fn configure_ngrok(token: &str) {
    let exe = if cfg!(target_os = "windows") { "ngrok.exe" } else { "./ngrok" };
    let _ = Command::new(exe).arg("config").arg("add-authtoken").arg(token).output();
}

fn cleanup_processes() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("taskkill").args(&["/F", "/IM", "ngrok.exe"]).output();
        let _ = Command::new("taskkill").args(&["/F", "/IM", "ssh.exe"]).output();
    } else {
        let _ = Command::new("pkill").arg("ngrok").output();
    }
}

fn download_ngrok() -> Result<(), Box<dyn std::error::Error>> {
    // EXHAUSTIVE OS LIST
    let target = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86")    => "windows-386",
        ("windows", _)        => "windows-amd64",
        ("macos", "aarch64")  => "darwin-arm64",
        ("macos", _)          => "darwin-amd64",
        ("linux", "aarch64")  => "linux-arm64",  // Rpi 4/5
        ("linux", "arm")      => "linux-arm",    // Rpi Zero
        ("linux", _)          => "linux-amd64",
        ("freebsd", "x86")    => "freebsd-386",
        ("freebsd", _)        => "freebsd-amd64",
        _ => return Err("OS not auto-supported. Use SSH fallback.".into()), 
    };

    let url = format!("https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip", target);
    println!("⬇️  Target: {}", target);
    
    let resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() { return Err("Download failed".into()); }
    
    let bytes = resp.bytes()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().contains("ngrok") {
            let mut out = File::create(NGROK_BIN)?;
            io::copy(&mut file, &mut out)?;
            return Ok(());
        }
    }
    Err("Ngrok binary not found".into())
}