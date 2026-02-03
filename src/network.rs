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
            loop {
                if let Ok((mut stream, addr)) = listener.accept() {
                    println!("✨ NEW PLAYER CONNECTED: {:?}", addr);
                    stream.set_nonblocking(false).unwrap(); 
                    
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

                    // Writer
                    let rx_out_thread = rx_out.clone();
                    thread::spawn(move || {
                        while let Ok(packet) = rx_out_thread.recv() {
                            let encoded = bincode::serialize(&packet).unwrap();
                            if stream_clone.write_all(&encoded).is_err() { break; }
                        }
                    });
                    break; // One client for now
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
        let stream = TcpStream::connect(ip).expect("Failed to connect");
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