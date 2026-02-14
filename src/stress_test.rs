use crate::network::Packet;
use std::thread;
use std::time::{Duration, Instant};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StressTestConfig {
    pub num_clients: usize,
    pub connection_delay_ms: u64,
    pub movement_frequency_hz: f64,
    pub block_update_frequency_hz: f64,
    pub test_duration_seconds: u64,
    pub server_address: String,
    pub username_prefix: String,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            num_clients: 10,
            connection_delay_ms: 100,
            movement_frequency_hz: 5.0,
            block_update_frequency_hz: 2.0,
            test_duration_seconds: 60,
            server_address: "127.0.0.1:8080".to_string(),
            username_prefix: "StressBot".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientMetrics {
    pub client_id: usize,
    pub connected: bool,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_time: Option<Instant>,
    pub last_activity: Option<Instant>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StressTestResults {
    pub config: StressTestConfig,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub client_metrics: Vec<ClientMetrics>,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub successful_connections: usize,
    pub failed_connections: usize,
}

pub struct StressTestManager {
    config: StressTestConfig,
    results: Arc<Mutex<StressTestResults>>,
    running: Arc<Mutex<bool>>,
    client_handles: Vec<thread::JoinHandle<()>>,
}

impl StressTestManager {
    pub fn new(config: StressTestConfig) -> Self {
        let results = Arc::new(Mutex::new(StressTestResults {
            config: config.clone(),
            start_time: Instant::now(),
            end_time: None,
            client_metrics: Vec::new(),
            total_packets_sent: 0,
            total_packets_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            successful_connections: 0,
            failed_connections: 0,
        }));
        
        Self {
            config,
            results,
            running: Arc::new(Mutex::new(false)),
            client_handles: Vec::new(),
        }
    }
    
    pub fn start_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("Test is already running".into());
        }
        *running = true;
        drop(running);
        
        // Clear previous results
        {
            let mut results = self.results.lock().unwrap();
            results.start_time = Instant::now();
            results.end_time = None;
            results.client_metrics.clear();
            results.total_packets_sent = 0;
            results.total_packets_received = 0;
            results.total_bytes_sent = 0;
            results.total_bytes_received = 0;
            results.successful_connections = 0;
            results.failed_connections = 0;
        }
        
        println!("🚀 Starting stress test with {} clients", self.config.num_clients);
        
        // Spawn client threads
        for i in 0..self.config.num_clients {
            let config = self.config.clone();
            let results = Arc::clone(&self.results);
            let running = Arc::clone(&self.running);
            let delay_ms = config.connection_delay_ms;
            
            let handle = thread::spawn(move || {
                Self::run_client(i, config, results, running);
            });
            
            self.client_handles.push(handle);
            
            // Stagger connections
            if i < self.config.num_clients - 1 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        
        Ok(())
    }
    
    pub fn stop_test(&mut self) -> StressTestResults {
        let mut running = self.running.lock().unwrap();
        *running = false;
        drop(running);
        
        // Wait for all clients to finish
        for handle in self.client_handles.drain(..) {
            let _ = handle.join();
        }
        
        let mut results = self.results.lock().unwrap();
        results.end_time = Some(Instant::now());
        results.clone()
    }
    
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    
    pub fn get_results(&self) -> StressTestResults {
        self.results.lock().unwrap().clone()
    }
    
    fn run_client(
        client_id: usize,
        config: StressTestConfig,
        results: Arc<Mutex<StressTestResults>>,
        running: Arc<Mutex<bool>>,
    ) {
        let mut metrics = ClientMetrics {
            client_id,
            connected: false,
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connection_time: None,
            last_activity: None,
            errors: Vec::new(),
        };
        
        // Attempt to connect
        match std::net::TcpStream::connect(&config.server_address) {
            Ok(mut stream) => {
                metrics.connected = true;
                metrics.connection_time = Some(Instant::now());
                metrics.last_activity = Some(Instant::now());
                
                // Send handshake
                let username = format!("{}{}", config.username_prefix, client_id);
                let handshake = Packet::Handshake { 
                    username: username.clone(), 
                    seed: 12345 + client_id as u32 
                };
                
                if let Err(e) = Self::send_packet(&mut stream, &handshake) {
                    metrics.errors.push(format!("Failed to send handshake: {}", e));
                    return;
                }
                
                let start_time = Instant::now();
                let mut last_movement = start_time;
                let mut last_block_update = start_time;
                let mut movement_counter = 0;
                let mut block_counter = 0;
                
                while *running.lock().unwrap() {
                    let now = Instant::now();
                    
                    // Send movement updates
                    if now.duration_since(last_movement).as_secs_f64() >= 1.0 / config.movement_frequency_hz {
                        let move_packet = Packet::PlayerMove {
                            id: client_id as u32,
                            x: (movement_counter as f32 * 10.0).sin() * 50.0,
                            y: 64.0 + (movement_counter as f32 * 0.1).sin() * 5.0,
                            z: (movement_counter as f32 * 10.0).cos() * 50.0,
                            ry: (movement_counter as f32 * 0.05) * std::f32::consts::PI * 2.0,
                        };
                        
                        if let Err(e) = Self::send_packet(&mut stream, &move_packet) {
                            metrics.errors.push(format!("Failed to send movement: {}", e));
                            break;
                        }
                        
                        metrics.packets_sent += 1;
                        last_movement = now;
                        movement_counter += 1;
                    }
                    
                    // Send block updates
                    if now.duration_since(last_block_update).as_secs_f64() >= 1.0 / config.block_update_frequency_hz {
                        let block_packet = Packet::BlockUpdate {
                            pos: crate::world::BlockPos {
                                x: (block_counter as i32 % 100) - 50,
                                y: 64,
                                z: ((block_counter / 100) as i32 % 100) - 50,
                            },
                            block: crate::world::BlockType::Stone,
                        };
                        
                        if let Err(e) = Self::send_packet(&mut stream, &block_packet) {
                            metrics.errors.push(format!("Failed to send block update: {}", e));
                            break;
                        }
                        
                        metrics.packets_sent += 1;
                        last_block_update = now;
                        block_counter += 1;
                    }
                    
                    // Try to receive data
                    match Self::try_receive_packet(&mut stream) {
                        Ok(Some(_packet)) => {
                            metrics.packets_received += 1;
                            metrics.last_activity = Some(Instant::now());
                        }
                        Ok(None) => {
                            // No data available
                        }
                        Err(e) => {
                            metrics.errors.push(format!("Receive error: {}", e));
                            break;
                        }
                    }
                    
                    thread::sleep(Duration::from_millis(10));
                }
                
                // Send disconnect
                let _ = Self::send_packet(&mut stream, &Packet::Disconnect);
            }
            Err(e) => {
                metrics.errors.push(format!("Connection failed: {}", e));
            }
        }
        
        // Update results
        {
            let mut results = results.lock().unwrap();
            results.client_metrics.push(metrics);
            
            if results.client_metrics[client_id].connected {
                results.successful_connections += 1;
            } else {
                results.failed_connections += 1;
            }
        }
    }
    
    fn send_packet(stream: &mut std::net::TcpStream, packet: &Packet) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(packet)?;
        let len = data.len() as u32;
        
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&data)?;
        stream.flush()?;
        
        Ok(())
    }
    
    fn try_receive_packet(stream: &mut std::net::TcpStream) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
        stream.set_nonblocking(true)?;
        
        let mut len_bytes = [0u8; 4];
        match stream.read_exact(&mut len_bytes) {
            Ok(_) => {
                let len = u32::from_le_bytes(len_bytes) as usize;
                if len > 65536 { // Reasonable max packet size
                    return Err("Packet too large".into());
                }
                
                let mut data = vec![0u8; len];
                stream.read_exact(&mut data)?;
                
                let packet: Packet = bincode::deserialize(&data)?;
                Ok(Some(packet))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl StressTestResults {
    pub fn print_summary(&self) {
        let duration = self.end_time.unwrap_or(Instant::now()).duration_since(self.start_time);
        let duration_secs = duration.as_secs_f32();
        
        println!("\n📊 STRESS TEST RESULTS");
        println!("====================");
        println!("Test Duration: {:.2} seconds", duration_secs);
        println!("Target Clients: {}", self.config.num_clients);
        println!("Successful Connections: {}", self.successful_connections);
        println!("Failed Connections: {}", self.failed_connections);
        println!("Success Rate: {:.1}%", (self.successful_connections as f32 / self.config.num_clients as f32) * 100.0);
        
        if self.successful_connections > 0 {
            println!("\n📈 NETWORK STATISTICS:");
            println!("Total Packets Sent: {}", self.total_packets_sent);
            println!("Total Packets Received: {}", self.total_packets_received);
            println!("Total Bytes Sent: {} KB", self.total_bytes_sent / 1024);
            println!("Total Bytes Received: {} KB", self.total_bytes_received / 1024);
            println!("Packets/sec: {:.1}", self.total_packets_sent as f32 / duration_secs);
            println!("Throughput: {:.1} KB/s", (self.total_bytes_sent + self.total_bytes_received) as f32 / duration_secs / 1024.0);
        }
        
        if !self.client_metrics.is_empty() {
            println!("\n🔍 CLIENT DETAILS:");
            for metrics in &self.client_metrics {
                let status = if metrics.connected { "✅ Connected" } else { "❌ Failed" };
                println!("  Client {}: {} (Sent: {}, Recv: {}, Errors: {})", 
                         metrics.client_id, status, metrics.packets_sent, metrics.packets_received, metrics.errors.len());
                
                if !metrics.errors.is_empty() {
                    for error in &metrics.errors {
                        println!("    Error: {}", error);
                    }
                }
            }
        }
    }
}
