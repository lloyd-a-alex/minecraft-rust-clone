use std::fs::{self, File};
use std::io::{self, Cursor, Write, BufRead, BufReader, Read}; // Added Read trait
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::env;

#[cfg(target_os = "windows")] const NGROK_BIN: &str = "ngrok.exe";
#[cfg(not(target_os = "windows"))] const NGROK_BIN: &str = "ngrok";

pub fn start_ngrok_tunnel(port: &str) -> Option<String> {
    let os = env::consts::OS.to_uppercase();
    println!("----------------------------------------------------------");
    println!("🔌 NETWORK INIT | OS: {}", os);

    cleanup_processes();

    // 2. NGROK ATTEMPT
    println!("🚀 Attempting Ngrok Tunnel...");
    println!("   (Press ENTER to skip if stuck)");
    if let Some(url) = attempt_ngrok(port) {
        return Some(url);
    }

    // 3. SSH FALLBACK
    println!("⚠️  Ngrok failed/skipped. Attempting SSH Tunnel...");
    println!("   (Press ENTER to skip to LAN Only)");
    attempt_ssh_tunnel(port)
}

fn attempt_ngrok(port: &str) -> Option<String> {
    // Check for token
    let token_file = Path::new("ngrok_token.txt");
    if token_file.exists() {
        if let Ok(token) = fs::read_to_string(token_file) {
            configure_ngrok(token.trim());
        }
    }

    let path = Path::new(NGROK_BIN);
    if !path.exists() {
        println!("⬇️  Ngrok binary missing. Downloading...");
        if let Err(_) = download_ngrok() {
            println!("❌ Download failed.");
            return None; 
        }
    }

    // Launch with piped stderr so we can see why it crashes
    let exe = if cfg!(target_os = "windows") { "ngrok.exe" } else { "./ngrok" };
    let mut child = Command::new(exe)
        .arg("tcp")
        .arg(port)
        .stdout(Stdio::null()) 
        .stderr(Stdio::piped()) // Capture errors!
        .spawn()
        .ok()?;

    // Input listener for SKIP
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
        let _ = tx.send(());
    });

    for _ in 0..30 { // 15 seconds max wait
        // Check Skip
        if rx.try_recv().is_ok() {
            println!("⏩ User skipped Ngrok.");
            let _ = child.kill();
            return None;
        }

        // Check Crash
        if let Ok(Some(_)) = child.try_wait() {
            // Read the error
            if let Some(mut stderr) = child.stderr.take() {
                let mut err_msg = String::new();
                let _ = stderr.read_to_string(&mut err_msg); // Fixed: read_to_string now works
                println!("❌ Ngrok crashed! Error output:\n{}", err_msg);
                
                if err_msg.contains("authentication failed") || err_msg.contains("authtoken") {
                    return handle_ngrok_auth(port);
                }
            }
            return None;
        }

        // Check API for Success
        if let Ok(resp) = reqwest::blocking::get("http://127.0.0.1:4040/api/tunnels") {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(url) = json["tunnels"][0]["public_url"].as_str() {
                    let clean = url.replace("tcp://", "");
                    println!("✅ NGROK CONNECTED: {}", clean);
                    return Some(clean.to_string());
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    
    let _ = child.kill();
    None
}

fn attempt_ssh_tunnel(port: &str) -> Option<String> {
    let mut child = Command::new("ssh")
        .arg("-R")
        .arg(format!("80:localhost:{}", port))
        .arg("-o").arg("StrictHostKeyChecking=no")
        .arg("nokey@localhost.run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    // Output Reader Thread (Non-blocking)
    let (tx_url, rx_url) = mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                if l.contains(".localhost.run") {
                     if let Some(start) = l.find("https://") {
                         let url = l[start..].trim().to_string();
                         let _ = tx_url.send(url.replace("https://", ""));
                         return;
                     }
                }
            }
        }
    });

    // Skip Listener
    let (tx_skip, rx_skip) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
        let _ = tx_skip.send(());
    });

    println!("🔄 Waiting for SSH... (Press ENTER to skip)");
    
    for _ in 0..40 { // 20 seconds
        if rx_skip.try_recv().is_ok() {
            println!("⏩ User skipped SSH.");
            let _ = child.kill();
            return None;
        }
        
        if let Ok(url) = rx_url.try_recv() {
            println!("✅ SSH TUNNEL CONNECTED: {}", url);
            return Some(url);
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    println!("❌ SSH Timed out.");
    let _ = child.kill();
    None
}

fn handle_ngrok_auth(port: &str) -> Option<String> {
    println!("🔑 NGROK AUTH NEEDED!");
    println!("   Go to: https://dashboard.ngrok.com/get-started/your-authtoken");
    println!("   Paste token below and hit ENTER:");
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
    }
}

fn download_ngrok() -> Result<(), Box<dyn std::error::Error>> {
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
        _ => return Err("OS not supported".into()), 
    };

    let url = format!("https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip", target);
    println!("⬇️  Target: {}", target);
    
    let resp = reqwest::blocking::get(url)?;
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
    Err("Not found".into())
}