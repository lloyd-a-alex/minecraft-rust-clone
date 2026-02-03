use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::world::{BlockPos, BlockType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Packet {
    // We add 'seed' here so the Client knows what world to generate
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
}

impl NetworkManager {
pub fn host(port: String) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out) = unbounded();
        
        let address = format!("0.0.0.0:{}", port);
        println!("🔥 HOSTING SERVER ON: {}", address);
        
        let listener = TcpListener::bind(&address).expect("Failed to bind to port");
        listener.set_nonblocking(true).unwrap();

        // Server Accept Thread
        let tx_in_clone = tx_in.clone();
        
        thread::spawn(move || {
            // We use a shared list of writers to broadcast packets to ALL clients
            // For this simple version, we just let them connect and spam the main thread.
            // A more complex server would need a list of clients protected by a Mutex.
            
            let mut client_id_counter = 2; // Host is 1, next is 2

            loop {
                if let Ok((mut stream, addr)) = listener.accept() {
                    println!("✨ NEW PLAYER CONNECTED: {:?} (ID: {})", addr, client_id_counter);
                    let _ = stream.set_nonblocking(false); 
                    
                    let mut stream_clone = stream.try_clone().unwrap();
                    let tx_in_thread = tx_in_clone.clone();
                    
                    // Client Reader
                    thread::spawn(move || {
                        let mut buffer = [0u8; 1024];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => break, // Disconnect
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

                    // Client Writer (Broadcasts everything to everyone - simplified)
                    // Note: In a real architecture, you'd want individual channels. 
                    // For this fix, we are just ensuring they CONNECT.
                    let rx_out_thread = rx_out.clone();
                    thread::spawn(move || {
                        while let Ok(packet) = rx_out_thread.recv() {
                            let encoded = bincode::serialize(&packet).unwrap();
                            if stream_clone.write_all(&encoded).is_err() { break; }
                        }
                    });
                    
                    client_id_counter += 1;
                    // REMOVED THE BREAK HERE. NOW ACCEPTS INFINITE PLAYERS.
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
        }
    }

pub fn join(ip: String) -> Self {
        let (tx_in, rx_in) = unbounded();
        let (tx_out, rx_out) = unbounded();

        println!("🚀 CONNECTING TO: {}", ip);
        
        // --- RETRY LOGIC START ---
        let start = std::time::Instant::now();
        let stream = loop {
            match TcpStream::connect(&ip) {
                Ok(s) => break s,
                Err(_) => {
                    if start.elapsed().as_secs() > 15 {
                        panic!("❌ CONNECTION TIMED OUT: Could not find server at {}", ip);
                    }
                    // Print a dot to show we are waiting
                    print!("."); 
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        };
        println!("\n✅ CONNECTED!");
        // --- RETRY LOGIC END ---

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
        }
    }

    pub fn send_packet(&self, packet: Packet) {
        let _ = self.sender.send(packet);
    }

    pub fn try_recv(&self) -> Option<Packet> {
        self.receiver.try_recv().ok()
    }
}