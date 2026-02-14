use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use glam::Vec3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_name: String,
    pub skin: PlayerSkin,
    pub nametag_color: [u8; 3], // RGB color
    pub show_nametag: bool,
    pub custom_cape: Option<String>, // Cape texture name
    pub achievements: Vec<String>,
    pub statistics: PlayerStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSkin {
    pub skin_type: SkinType,
    pub base_color: [u8; 3], // RGB
    pub hair_color: [u8; 3], // RGB
    pub eye_color: [u8; 3], // RGB
    pub shirt_color: [u8; 3], // RGB
    pub pants_color: [u8; 3], // RGB
    pub shoes_color: [u8; 3], // RGB
    pub accessories: Vec<String>, // Hat, glasses, etc.
    pub custom_texture: Option<String>, // Custom texture path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkinType {
    Steve, // Male
    Alex, // Female
    Custom, // Custom model
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    pub blocks_placed: u64,
    pub blocks_broken: u64,
    pub mobs_killed: u64,
    pub deaths: u64,
    pub distance_walked: f64,
    pub time_played: f64,
    pub items_crafted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nametag {
    pub player_name: String,
    pub position: Vec3,
    pub color: [u8; 3],
    pub visible: bool,
    pub offset_y: f32, // Height above player head
}

pub struct PlayerCustomizationSystem {
    pub profiles: HashMap<String, PlayerProfile>,
    pub current_profile: Option<String>,
    pub available_skins: Vec<SkinType>,
    pub available_colors: Vec<[u8; 3]>,
    pub nametags: Vec<Nametag>,
}

impl PlayerCustomizationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            profiles: HashMap::new(),
            current_profile: None,
            available_skins: vec![SkinType::Steve, SkinType::Alex],
            available_colors: vec![
                [255, 255, 255], // White
                [0, 0, 0], // Black
                [255, 0, 0], // Red
                [0, 255, 0], // Green
                [0, 0, 255], // Blue
                [255, 255, 0], // Yellow
                [255, 0, 255], // Magenta
                [0, 255, 255], // Cyan
                [128, 128, 128], // Gray
                [165, 42, 42], // Brown
                [255, 165, 0], // Orange
                [128, 0, 128], // Purple
                [0, 128, 128], // Teal
            ],
            nametags: Vec::new(),
        };

        // Create default profile
        let default_profile = PlayerProfile {
            player_name: "Player".to_string(),
            skin: PlayerSkin {
                skin_type: SkinType::Steve,
                base_color: [255, 220, 177], // Light skin tone
                hair_color: [139, 69, 19], // Brown
                eye_color: [0, 0, 0], // Black
                shirt_color: [0, 0, 255], // Blue shirt
                pants_color: [0, 0, 139], // Dark blue pants
                shoes_color: [0, 0, 0], // Black shoes
                accessories: Vec::new(),
                custom_texture: None,
            },
            nametag_color: [255, 255, 255], // White
            show_nametag: true,
            custom_cape: None,
            achievements: Vec::new(),
            statistics: PlayerStatistics {
                blocks_placed: 0,
                blocks_broken: 0,
                mobs_killed: 0,
                deaths: 0,
                distance_walked: 0.0,
                time_played: 0.0,
                items_crafted: 0,
            },
        };

        system.profiles.insert("default".to_string(), default_profile);
        system.current_profile = Some("default".to_string());

        system
    }

    pub fn create_profile(&mut self, name: &str, profile: PlayerProfile) -> Result<(), String> {
        if self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' already exists", name));
        }

        self.profiles.insert(name.to_string(), profile);
        Ok(())
    }

    pub fn get_current_profile(&self) -> Option<&PlayerProfile> {
        if let Some(ref profile_name) = self.current_profile {
            self.profiles.get(profile_name)
        } else {
            None
        }
    }

    pub fn get_current_profile_mut(&mut self) -> Option<&mut PlayerProfile> {
        if let Some(ref profile_name) = self.current_profile {
            self.profiles.get_mut(profile_name)
        } else {
            None
        }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Result<(), String> {
        if !self.profiles.contains_key(profile_name) {
            return Err(format!("Profile '{}' does not exist", profile_name));
        }

        self.current_profile = Some(profile_name.to_string());
        Ok(())
    }

    pub fn update_player_name(&mut self, name: &str) -> Result<(), String> {
        if name.is_empty() || name.len() > 16 {
            return Err("Name must be between 1 and 16 characters".to_string());
        }

        if let Some(profile) = self.get_current_profile_mut() {
            profile.player_name = name.to_string();
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_skin_color(&mut self, color_type: SkinColorType, color: [u8; 3]) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            match color_type {
                SkinColorType::Base => profile.skin.base_color = color,
                SkinColorType::Hair => profile.skin.hair_color = color,
                SkinColorType::Eyes => profile.skin.eye_color = color,
                SkinColorType::Shirt => profile.skin.shirt_color = color,
                SkinColorType::Pants => profile.skin.pants_color = color,
                SkinColorType::Shoes => profile.skin.shoes_color = color,
            }
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_nametag_color(&mut self, color: [u8; 3]) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.nametag_color = color;
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn toggle_nametag(&mut self) -> Result<bool, String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.show_nametag = !profile.show_nametag;
            Ok(profile.show_nametag)
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn add_accessory(&mut self, accessory: String) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            if !profile.skin.accessories.contains(&accessory) {
                profile.skin.accessories.push(accessory);
            }
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn remove_accessory(&mut self, accessory: &str) -> Result<(), String> {
        if let Some(profile) = self.get_current_profile_mut() {
            profile.skin.accessories.retain(|a| a != accessory);
            Ok(())
        } else {
            Err("No profile selected".to_string())
        }
    }

    pub fn update_nametags(&mut self, player_positions: &[(String, Vec3)]) {
        self.nametags.clear();
        
        for (player_name, position) in player_positions {
            if let Some(profile) = self.profiles.get(player_name) {
                if profile.show_nametag {
                    let nametag = Nametag {
                        player_name: profile.player_name.clone(),
                        position: *position + Vec3::new(0.0, 1.8, 0.0), // Above player head
                        color: profile.nametag_color,
                        visible: true,
                        offset_y: 0.3,
                    };
                    self.nametags.push(nametag);
                }
            }
        }
    }

    pub fn get_nametag_at_position(&self, position: Vec3, view_direction: Vec3, max_distance: f32) -> Option<&Nametag> {
        for nametag in &self.nametags {
            let distance = (nametag.position - position).length();
            if distance <= max_distance {
                // Check if player is looking at this nametag
                let to_nametag = nametag.position - position;
                let dot = view_direction.dot(to_nametag.normalize());
                if dot > 0.7 { // Within ~45 degree cone
                    return Some(nametag);
                }
            }
        }
        None
    }

    pub fn get_all_nametags_in_range(&self, position: Vec3, max_distance: f32) -> Vec<&Nametag> {
        self.nametags
            .iter()
            .filter(|nametag| (nametag.position - position).length() <= max_distance)
            .collect()
    }

    pub fn save_profiles(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.profiles)
    }

    pub fn load_profiles(&mut self, json_data: &str) -> Result<(), serde_json::Error> {
        self.profiles = serde_json::from_str(json_data)?;
        Ok(())
    }

    pub fn get_available_colors(&self) -> &[[u8; 3]] {
        &self.available_colors
    }

    pub fn get_available_skins(&self) -> &[SkinType] {
        &self.available_skins
    }

    pub fn get_available_accessories(&self) -> Vec<String> {
        vec![
            "None".to_string(),
            "Baseball Cap".to_string(),
            "Crown".to_string(),
            "Glasses".to_string(),
            "Headband".to_string(),
            "Mining Helmet".to_string(),
            "Sunglasses".to_string(),
            "Top Hat".to_string(),
            "Witch Hat".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SkinColorType {
    Base,
    Hair,
    Eyes,
    Shirt,
    Pants,
    Shoes,
}

impl PlayerCustomizationSystem {
    pub fn get_skin_texture_coords(&self, profile: &PlayerProfile) -> [u32; 6] {
        // Return texture coordinates for different skin parts
        // This would be used by the renderer to draw the custom skin
        match profile.skin.skin_type {
            SkinType::Steve => [0, 1, 2, 3, 4, 5], // Default Steve textures
            SkinType::Alex => [6, 7, 8, 9, 10, 11], // Default Alex textures
            SkinType::Custom => [12, 13, 14, 15, 16, 17], // Custom texture
        }
    }

    pub fn get_skin_colors(&self, profile: &PlayerProfile) -> [[u8; 3]; 6] {
        [
            profile.skin.base_color,
            profile.skin.hair_color,
            profile.skin.eye_color,
            profile.skin.shirt_color,
            profile.skin.pants_color,
            profile.skin.shoes_color,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinPreset {
    pub name: String,
    pub skin: PlayerSkin,
    pub description: String,
}

impl PlayerCustomizationSystem {
    pub fn get_default_presets(&self) -> Vec<SkinPreset> {
        vec![
            SkinPreset {
                name: "Default Steve".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Steve,
                    base_color: [255, 220, 177],
                    hair_color: [139, 69, 19],
                    eye_color: [0, 0, 0],
                    shirt_color: [0, 0, 255],
                    pants_color: [0, 0, 139],
                    shoes_color: [0, 0, 0],
                    accessories: Vec::new(),
                    custom_texture: None,
                },
                description: "Classic Steve skin".to_string(),
            },
            SkinPreset {
                name: "Default Alex".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Alex,
                    base_color: [255, 220, 177],
                    hair_color: [255, 140, 0], // Blonde hair
                    eye_color: [0, 100, 0], // Green eyes
                    shirt_color: [255, 0, 0], // Red shirt
                    pants_color: [128, 0, 128], // Purple pants
                    shoes_color: [139, 69, 19], // Brown shoes
                    accessories: vec!["Baseball Cap".to_string()],
                    custom_texture: None,
                },
                description: "Classic Alex skin with baseball cap".to_string(),
            },
            SkinPreset {
                name: "Miner".to_string(),
                skin: PlayerSkin {
                    skin_type: SkinType::Steve,
                    base_color: [139, 69, 19], // Darker skin
                    hair_color: [0, 0, 0], // Black hair
                    eye_color: [139, 69, 19], // Brown eyes
                    shirt_color: [128, 128, 128], // Gray shirt
                    pants_color: [64, 64, 64], // Dark gray pants
                    shoes_color: [0, 0, 0], // Black shoes
                    accessories: vec!["Mining Helmet".to_string()],
                    custom_texture: None,
                },
                description: "Professional miner outfit".to_string(),
            },
        ]
    }

    pub fn apply_preset(&mut self, preset_name: &str) -> Result<(), String> {
        let presets = self.get_default_presets();
        
        if let Some(preset) = presets.iter().find(|p| p.name == preset_name) {
            if let Some(profile) = self.get_current_profile_mut() {
                profile.skin = preset.skin.clone();
                Ok(())
            } else {
                Err("No profile selected".to_string())
            }
        } else {
            Err(format!("Preset '{}' not found", preset_name))
        }
    }
}
