use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::world::World;
use crate::player::Player;
use glam::Vec3;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveData {
    pub version: u32,
    pub seed: u32,
    pub player_position: Vec3,
    pub player_rotation: Vec3,
    pub player_health: f32,
    pub player_inventory: Vec<(u32, u8)>, // (block_id, count)
    pub world_chunks: HashMap<(i32, i32, i32), ChunkData>,
    pub play_time: f64, // Total play time in seconds
    pub last_saved: String, // ISO timestamp
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkData {
    pub blocks: Vec<u8>, // Compressed block data
    pub is_empty: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveSlot {
    pub name: String,
    pub world_name: String,
    pub created: String,
    pub modified: String,
    pub play_time: f64,
    pub screenshot: Option<String>, // Base64 encoded screenshot
    pub data: Option<SaveData>,
}

impl SaveSlot {
    pub fn new(slot_num: usize) -> Self {
        Self {
            name: format!("Save Slot {}", slot_num + 1),
            world_name: format!("World {}", slot_num + 1),
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
            play_time: 0.0,
            screenshot: None,
            data: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
    
    pub fn update_modified(&mut self) {
        self.modified = chrono::Utc::now().to_rfc3339();
    }
}

pub struct SaveManager {
    pub slots: Vec<SaveSlot>,
    pub current_slot: Option<usize>,
    pub auto_save_interval: f64, // in seconds
    pub last_auto_save: f64,
}

impl SaveManager {
    pub fn new() -> Self {
        let mut slots = Vec::new();
        for i in 0..5 {
            slots.push(SaveSlot::new(i));
        }
        
        Self {
            slots,
            current_slot: None,
            auto_save_interval: 300.0, // 5 minutes
            last_auto_save: 0.0,
        }
    }
    
    pub fn load_saves(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let saves_path = "saves/";
        std::fs::create_dir_all(saves_path)?;
        
        for i in 0..5 {
            let file_path = format!("saves/save_slot_{}.json", i);
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(slot) = serde_json::from_str::<SaveSlot>(&content) {
                    self.slots[i] = slot;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn save_slot(&mut self, slot_index: usize, world: &World, player: &Player, play_time: f64) -> Result<(), Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        // Compress world data
        let mut world_chunks = HashMap::new();
        for (&chunk_pos, chunk) in &world.chunks {
            let mut blocks = Vec::new();
            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        blocks.push(chunk.blocks[x][y][z] as u8);
                    }
                }
            }
            
            world_chunks.insert(chunk_pos, ChunkData {
                blocks,
                is_empty: chunk.is_empty,
            });
        }
        
        // Convert inventory
        let mut player_inventory = Vec::new();
        for (_i, slot) in player.inventory.slots.iter().enumerate() {
            if let Some(stack) = slot {
                player_inventory.push((stack.item as u32, stack.count));
            }
        }
        
        let save_data = SaveData {
            version: 1,
            seed: world.seed,
            player_position: player.position,
            player_rotation: player.rotation,
            player_health: player.health,
            player_inventory,
            world_chunks,
            play_time,
            last_saved: chrono::Utc::now().to_rfc3339(),
        };
        
        self.slots[slot_index].data = Some(save_data);
        self.slots[slot_index].update_modified();
        self.slots[slot_index].play_time = play_time;
        
        // Write to file
        let file_path = format!("saves/save_slot_{}.json", slot_index);
        let content = serde_json::to_string_pretty(&self.slots[slot_index])?;
        std::fs::write(file_path, content)?;
        
        Ok(())
    }
    
    pub fn load_slot(&mut self, slot_index: usize) -> Result<SaveData, Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        if let Some(ref data) = self.slots[slot_index].data {
            Ok(data.clone())
        } else {
            Err("Save slot is empty".into())
        }
    }
    
    pub fn delete_slot(&mut self, slot_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if slot_index >= self.slots.len() {
            return Err("Invalid slot index".into());
        }
        
        self.slots[slot_index] = SaveSlot::new(slot_index);
        
        let file_path = format!("saves/save_slot_{}.json", slot_index);
        let _ = std::fs::remove_file(file_path);
        
        Ok(())
    }
    
    pub fn should_auto_save(&mut self, current_time: f64) -> bool {
        if current_time - self.last_auto_save >= self.auto_save_interval {
            self.last_auto_save = current_time;
            true
        } else {
            false
        }
    }
}
