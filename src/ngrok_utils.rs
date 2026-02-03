use std::fs::{self, File};
use std::io::{self, Cursor, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::env;

#[cfg(target_os = "windows")] const NGROK_BIN: &str = "ngrok.exe";
#[cfg(not(target_os = "windows"))] const NGROK_BIN: &str = "ngrok";

pub fn start_ngrok_tunnel(port: &str) -> Option<String> {
    println!("----------------------------------------------------------");
    println!("🔌 INITIALIZING TUNNEL SYSTEM ({})", env::consts::OS.to_uppercase());

    kill_existing_ngrok();

    // 1. AUTO-AUTH (Check for saved token)
    let token_file = Path::new("ngrok_token.txt");
    if token_file.exists() {
        if let Ok(token) = fs::read_to_string(token_file) {
            let token = token.trim();
            if !token.is_empty() {
                println!("🔑 Found saved token. Authenticating...");
                configure_ngrok(token);
            }
        }
    }

    let path = Path::new(NGROK_BIN);
    if !path.exists() {
        println!("⬇️  Downloading ngrok...");
        if let Err(e) = download_ngrok() {
            println!("❌ Download failed: {}", e);
            return None;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
    }

    // Launch
    let log_file = File::create("ngrok.log").ok()?;
    let mut child = Command::new(format!("./{}", NGROK_BIN).replace("./ngrok.exe", "ngrok.exe"))
        .arg("tcp")
        .arg(port)
        .stdout(Stdio::from(log_file))
        .spawn()
        .ok()?;

    println!("⏳ Connecting to global network...");
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        if let Ok(Some(_)) = child.try_wait() {
            return handle_auth_failure(port);
        }
        if let Ok(resp) = reqwest::blocking::get("http://127.0.0.1:4040/api/tunnels") {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(url) = json["tunnels"][0]["public_url"].as_str() {
                    let clean = url.replace("tcp://", "");
                    println!("✅ SUCCESS! Share this:  {}  ", clean);
                    return Some(clean.to_string());
                }
            }
        }
    }
    let _ = child.kill();
    handle_auth_failure(port)
}

fn configure_ngrok(token: &str) {
    let exe = if cfg!(target_os = "windows") { "ngrok.exe" } else { "./ngrok" };
    let _ = Command::new(exe).arg("config").arg("add-authtoken").arg(token).output();
}

fn handle_auth_failure(port: &str) -> Option<String> {
    println!("----------------------------------------------------------");
    println!("🔑 AUTHENTICATION REQUIRED (First Time Only)");
    println!("1. Go to: https://dashboard.ngrok.com/get-started/your-authtoken");
    println!("2. Paste Token Below:");
    print!("> ");
    io::stdout().flush().unwrap();
    
    let mut token = String::new();
    io::stdin().read_line(&mut token).ok()?;
    let token = token.trim().to_string();
    
    // SAVE TOKEN
    let _ = fs::write("ngrok_token.txt", &token);
    configure_ngrok(&token);
    
    println!("🔄 Retrying...");
    start_ngrok_tunnel(port)
}

fn download_ngrok() -> Result<(), Box<dyn std::error::Error>> {
    let target = match (env::consts::OS, env::consts::ARCH) {
        ("windows", _) => "windows-amd64",
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", _) => "darwin-amd64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", "arm") => "linux-arm",
        ("linux", _) => "linux-amd64",
        _ => "linux-amd64", // Fallback
    };
    let url = format!("https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-{}.zip", target);
    let resp = reqwest::blocking::get(url)?.bytes()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(resp))?;
    let mut file = archive.by_index(0)?;
    let mut out = File::create(NGROK_BIN)?;
    io::copy(&mut file, &mut out)?;
    Ok(())
}

fn kill_existing_ngrok() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("taskkill").args(&["/F", "/IM", "ngrok.exe"]).output();
    } else {
        let _ = Command::new("pkill").arg("ngrok").output();
    }
}