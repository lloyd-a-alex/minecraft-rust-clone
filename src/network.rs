use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::world::{BlockPos, BlockType};

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
                    let rx_out_thread = rx_out.clone();
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
        let (tx_out, rx_out) = unbounded();

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