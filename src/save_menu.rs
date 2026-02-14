use crate::{MenuButton, MenuAction, Rect};
use crate::save_system::{SaveManager, SaveSlot};

#[derive(Clone)]
pub enum SaveMenuAction {
    Back,
    SelectSlot(usize),
    DeleteSlot(usize),
    CreateNew,
}

pub struct SaveMenu {
    pub buttons: Vec<MenuButton>,
    pub save_manager: SaveManager,
    pub selected_slot: Option<usize>,
    pub action: Option<SaveMenuAction>,
}

impl SaveMenu {
    pub fn new() -> Self {
        let mut save_manager = SaveManager::new();
        let _ = save_manager.load_saves(); // Load existing saves
        
        Self {
            buttons: Vec::new(),
            save_manager,
            selected_slot: None,
            action: None,
        }
    }
    
    pub fn update_buttons(&mut self) {
        self.buttons.clear();
        
        let w = 0.6; let h = 0.1; let g = 0.02; let start_y = 0.3;
        
        // Title
        self.buttons.push(MenuButton {
            rect: Rect { x: 0.0, y: 0.7, w: 0.4, h: 0.05 },
            text: "SELECT WORLD".to_string(),
            action: MenuAction::Resume, // Dummy action
            hovered: false,
        });
        
        // Save slots
        for i in 0..5 {
            let y = start_y - (i as f32) * (h + g);
            let slot = &self.save_manager.slots[i];
            
            let text = if slot.is_empty() {
                format!("Slot {} - [EMPTY]", i + 1)
            } else {
                format!("Slot {} - {} ({:.1}h)", i + 1, slot.world_name, slot.play_time / 3600.0)
            };
            
            self.buttons.push(MenuButton {
                rect: Rect { x: 0.0, y, w, h },
                text,
                action: MenuAction::Resume, // Will be handled specially
                hovered: false,
            });
        }
        
        // Back button
        self.buttons.push(MenuButton {
            rect: Rect { x: 0.0, y: -0.7, w: 0.3, h: 0.08 },
            text: "BACK".to_string(),
            action: MenuAction::Quit, // Use Quit as back
            hovered: false,
        });
    }
    
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<SaveMenuAction> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.rect.contains(x, y) {
                match i {
                    0 => return None, // Title, no action
                    6 => return Some(SaveMenuAction::Back), // Back button
                    slot_idx => {
                        let slot_num = slot_idx - 1; // Adjust for title button
                        if slot_num < 5 {
                            if self.save_manager.slots[slot_num].is_empty() {
                                return Some(SaveMenuAction::SelectSlot(slot_num));
                            } else {
                                // Could add confirmation dialog here
                                return Some(SaveMenuAction::SelectSlot(slot_num));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    pub fn get_slot_info(&self, slot: usize) -> Option<&SaveSlot> {
        if slot < 5 {
            Some(&self.save_manager.slots[slot])
        } else {
            None
        }
    }
}
