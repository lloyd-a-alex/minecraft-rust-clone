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

    // 2. NGROK ATTEMPT
    if let Some(url) = attempt_ngrok(port) {
        return Some(url);
    }

    // 3. FALLBACK: SSH TUNNEL (The "Universal" backup)
    println!("⚠️  Ngrok failed or unavailable. Attempting SSH Fallback...");
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
        if let Err(e) = download_ngrok() {
            println!("❌ Ngrok download not available for this OS: {}", e);
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

    println!("⏳ Starting Ngrok tunnel...");
    
    // Poll for Success (10s timeout)
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        
        // Did it crash? (Likely auth missing)
        if let Ok(Some(_)) = child.try_wait() {
            return handle_ngrok_auth(port);
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
    
    let _ = child.kill();
    None
}

fn attempt_ssh_tunnel(port: &str) -> Option<String> {
    println!("🔄 Trying 'localhost.run' via SSH...");
    
    // Command: ssh -R 80:localhost:PORT -o StrictHostKeyChecking=no nokey@localhost.run
    let mut child = Command::new("ssh")
        .arg("-R")
        .arg(format!("80:localhost:{}", port))
        .arg("-o").arg("StrictHostKeyChecking=no") // Don't ask for fingerprint
        .arg("nokey@localhost.run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // They often print info to stderr
        .spawn()
        .ok()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    // Scan output for 10 seconds to find the URL
    // Expected output: "Connect to your localhost app on https://random-name.localhost.run"
    let start = std::time::Instant::now();
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains(".localhost.run") {
                // Extract URL logic
                if let Some(start_idx) = l.find("https://") {
                    let url = l[start_idx..].trim().to_string();
                    let clean = url.replace("https://", ""); // Game assumes TCP/Address format mostly, but http is fine for display
                    println!("✅ SSH TUNNEL CONNECTED: {}", clean);
                    println!("ℹ️  (Note: This is an HTTP tunnel, might require Client tweaks for TCP)");
                    return Some(clean);
                }
            }
        }
        if start.elapsed().as_secs() > 10 { break; }
    }
    
    println!("❌ SSH Fallback failed. Ensure 'ssh' is in your PATH.");
    let _ = child.kill();
    None
}

fn handle_ngrok_auth(port: &str) -> Option<String> {
    println!("🔑 NGROK AUTH REQUIRED");
    println!("1. Copy token from: https://dashboard.ngrok.com/get-started/your-authtoken");
    println!("2. Paste here:");
    print!("> ");
    io::stdout().flush().unwrap();
    
    let mut token = String::new();
    if io::stdin().read_line(&mut token).is_ok() {
        let token = token.trim();
        let _ = fs::write("ngrok_token.txt", token);
        configure_ngrok(token);
        return attempt_ngrok(port);
    }
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
        // Don't pkill ssh on linux indiscriminately, might kill user session!
    }
}

// --- EXHAUSTIVE OS/ARCH DOWNLOADER ---
fn download_ngrok() -> Result<(), Box<dyn std::error::Error>> {
    let target = match (env::consts::OS, env::consts::ARCH) {
        // WINDOWS
        ("windows", "x86")    => "windows-386",
        ("windows", _)        => "windows-amd64",
        
        // MACOS
        ("macos", "aarch64")  => "darwin-arm64", // Apple Silicon
        ("macos", _)          => "darwin-amd64", // Intel
        
        // LINUX (Includes Docker, WSL)
        ("linux", "aarch64")  => "linux-arm64",  // Raspberry Pi 3/4/5
        ("linux", "arm")      => "linux-arm",    // Raspberry Pi Zero/1/2
        ("linux", "x86")      => "linux-386",
        ("linux", _)          => "linux-amd64",
        
        // FREEBSD (TrueNAS, Servers)
        ("freebsd", "x86")    => "freebsd-386",
        ("freebsd", _)        => "freebsd-amd64",
        
        // UNSUPPORTED BY NGROK DIRECTLY (Try Linux emulation or fallback)
        ("openbsd", _)        => return Err("OpenBSD not supported by Ngrok auto-downloader".into()),
        ("dragonfly", _)      => return Err("DragonFlyBSD not supported by Ngrok auto-downloader".into()),
        ("solaris", _)        => return Err("Solaris not supported by Ngrok auto-downloader".into()),
        
        _ => "linux-amd64", // Default fallback
    };

    let url = format!("https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip", target);
    println!("⬇️  Target: {}", target);
    
    let resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        return Err(format!("Download failed: {}", resp.status()).into());
    }
    
    let bytes = resp.bytes()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    
    // Find the executable in the zip (it might be named ngrok or ngrok.exe)
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().contains("ngrok") {
            let mut out = File::create(NGROK_BIN)?;
            io::copy(&mut file, &mut out)?;
            return Ok(());
        }
    }
    Err("Ngrok binary not found in zip".into())
}