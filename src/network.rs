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

impl NetworkManager {
    pub fn host(port: String, seed: u32) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out) = unbounded();
        
        let address = format!("0.0.0.0:{}", port);
        println!("🔥 HOSTING SERVER ON: {}", address);
        
        let listener = TcpListener::bind(&address).expect("Failed to bind to port");
        listener.set_nonblocking(true).unwrap();

        let tx_in_clone = tx_in.clone();
        
        thread::spawn(move || {
            let mut client_id_counter = 2; // Host is 1
            loop {
                if let Ok((mut stream, addr)) = listener.accept() {
println!("✨ NEW PLAYER CONNECTED: {:?} (ID: {})", addr, client_id_counter);
                    let _ = stream.set_nonblocking(false); 
                    
                    // --- INSTANT HANDSHAKE (SYNC WORLD) ---
                    let handshake = Packet::Handshake { username: "Host".to_string(), seed };
                    let bytes = bincode::serialize(&handshake).unwrap();
                    let _ = stream.write_all(&bytes);
                    // --------------------------------------

                    let mut stream_clone = stream.try_clone().unwrap();
                    let tx_in_thread = tx_in_clone.clone();
                    
                    // Reader
                    thread::spawn(move || {
                        let mut buffer = [0u8; 1024];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Ok(packet) = bincode::deserialize::<Packet>(&buffer[..n]) {
                                        tx_in_thread.send(packet).unwrap();
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
                            let encoded = bincode::serialize(&packet).unwrap();
                            if stream_clone.write_all(&encoded).is_err() { break; }
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

    pub fn join(ip: String) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out) = unbounded();

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
        println!("\n✅ CONNECTED!");

        let mut stream_read = stream.try_clone().unwrap();
        let mut stream_write = stream.try_clone().unwrap();

        // Reader
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match stream_read.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(packet) = bincode::deserialize::<Packet>(&buffer[..n]) {
                            tx_in.send(packet).unwrap();
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
                let encoded = bincode::serialize(&packet).unwrap();
                if stream_write.write_all(&encoded).is_err() { break; }
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