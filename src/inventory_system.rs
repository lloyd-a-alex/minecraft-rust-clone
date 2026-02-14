use crate::world::BlockType;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_type: BlockType,
    pub count: u8,
    pub durability: Option<u16>, // For tools and armor
}

impl ItemStack {
    pub fn new(item_type: BlockType, count: u8) -> Self {
        Self {
            item_type,
            count,
            durability: None,
        }
    }

    pub fn with_durability(item_type: BlockType, count: u8, durability: u16) -> Self {
        Self {
            item_type,
            count,
            durability: Some(durability),
        }
    }

    pub fn is_stackable(&self) -> bool {
        // Tools and armor are not stackable
        !self.item_type.is_tool() && self.count < 64
    }

    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.item_type == other.item_type && 
        self.is_stackable() && 
        other.is_stackable() &&
        self.durability.is_none() && other.durability.is_none()
    }

    pub fn merge(&mut self, other: &mut ItemStack) -> bool {
        if !self.can_merge_with(other) {
            return false;
        }

        let available_space = 64 - self.count;
        let transfer_amount = other.count.min(available_space);

        self.count += transfer_amount;
        other.count -= transfer_amount;

        if other.count == 0 {
            *other = ItemStack::new(BlockType::Air, 0);
        }

        true
    }

    pub fn split(&mut self, amount: u8) -> Option<ItemStack> {
        if amount == 0 || amount > self.count {
            return None;
        }

        let split_stack = ItemStack::new(self.item_type, amount);
        self.count -= amount;

        if self.count == 0 {
            *self = ItemStack::new(BlockType::Air, 0);
        }

        Some(split_stack)
    }

    pub fn is_empty(&self) -> bool {
        self.item_type == BlockType::Air || self.count == 0
    }

    pub fn get_max_stack_size(&self) -> u8 {
        if self.item_type.is_tool() {
            1
        } else {
            64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub hotbar_slots: usize,
    pub main_slots: usize,
}

impl Inventory {
    pub fn new(hotbar_slots: usize, main_slots: usize) -> Self {
        let total_slots = hotbar_slots + main_slots;
        let mut slots = Vec::with_capacity(total_slots);
        
        for _ in 0..total_slots {
            slots.push(None);
        }

        Self {
            slots,
            hotbar_slots,
            main_slots,
        }
    }

    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        if index < self.slots.len() {
            self.slots[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Option<ItemStack>> {
        if index < self.slots.len() {
            Some(&mut self.slots[index])
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) -> bool {
        if index < self.slots.len() {
            self.slots[index] = stack;
            true
        } else {
            false
        }
    }

    pub fn add_item(&mut self, mut item: ItemStack) -> bool {
        // Try to stack with existing items first
        for i in 0..self.slots.len() {
            if let Some(ref mut existing) = self.slots[i] {
                if existing.can_merge_with(&item) {
                    existing.merge(&mut item);
                    if item.is_empty() {
                        return true;
                    }
                }
            }
        }

        // Find empty slot
        for i in 0..self.slots.len() {
            if self.slots[i].is_none() || self.slots[i].as_ref().unwrap().is_empty() {
                self.slots[i] = Some(item);
                return true;
            }
        }

        false
    }

    pub fn remove_item(&mut self, index: usize, amount: u8) -> Option<ItemStack> {
        if let Some(ref mut stack) = self.slots[index] {
            stack.split(amount)
        } else {
            None
        }
    }

    pub fn swap_slots(&mut self, index1: usize, index2: usize) -> bool {
        if index1 < self.slots.len() && index2 < self.slots.len() {
            self.slots.swap(index1, index2);
            true
        } else {
            false
        }
    }

    pub fn get_first_empty_slot(&self) -> Option<usize> {
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.is_none() || slot.as_ref().unwrap().is_empty() {
                return Some(i);
            }
        }
        None
    }

    pub fn count_items(&self, item_type: BlockType) -> u32 {
        let mut count = 0;
        for slot in &self.slots {
            if let Some(stack) = slot {
                if stack.item_type == item_type {
                    count += stack.count as u32;
                }
            }
        }
        count
    }

    pub fn has_item(&self, item_type: BlockType) -> bool {
        self.count_items(item_type) > 0
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragOperation {
    pub source_slot: usize,
    pub source_stack: ItemStack,
    pub drag_slots: Vec<usize>,
    pub split_mode: SplitMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitMode {
    None,
    Even,
    Half,
    Custom(u8),
}

pub struct InventoryDragHandler {
    pub current_operation: Option<DragOperation>,
    pub drag_start_slot: Option<usize>,
    pub is_dragging: bool,
}

impl InventoryDragHandler {
    pub fn new() -> Self {
        Self {
            current_operation: None,
            drag_start_slot: None,
            is_dragging: false,
        }
    }

    pub fn start_drag(&mut self, inventory: &mut Inventory, slot_index: usize) -> bool {
        if let Some(stack) = inventory.get_slot(slot_index) {
            if !stack.is_empty() {
                self.drag_start_slot = Some(slot_index);
                self.is_dragging = true;
                self.current_operation = Some(DragOperation {
                    source_slot: slot_index,
                    source_stack: *stack,
                    drag_slots: vec![slot_index],
                    split_mode: SplitMode::None,
                });
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn update_drag(&mut self, inventory: &mut Inventory, slot_index: usize, split_mode: SplitMode) -> bool {
        if !self.is_dragging || self.drag_start_slot.is_none() {
            return false;
        }

        let source_slot = self.drag_start_slot.unwrap();
        if source_slot == slot_index {
            return false;
        }

        if let Some(ref mut operation) = self.current_operation {
            // Check if this slot is already in the drag path
            if operation.drag_slots.contains(&slot_index) {
                return false;
            }

            // Try to add this slot to the drag operation
            if let Some(source_stack) = inventory.get_slot(source_slot) {
                if let Some(target_stack) = inventory.get_slot(slot_index) {
                    // Check if we can merge
                    if source_stack.can_merge_with(target_stack) {
                        operation.drag_slots.push(slot_index);
                        operation.split_mode = split_mode;
                        return true;
                    }
                } else {
                    // Empty slot - can place item here
                    operation.drag_slots.push(slot_index);
                    operation.split_mode = split_mode;
                    return true;
                }
            }
        }

        false
    }

    pub fn end_drag(&mut self, inventory: &mut Inventory) -> bool {
        if !self.is_dragging || self.current_operation.is_none() {
            return false;
        }

        let operation = self.current_operation.take().unwrap();
        let source_slot = operation.source_slot;

        if let Some(source_stack) = inventory.remove_item(source_slot, operation.source_stack.count) {
            let mut remaining_stack = source_stack;

            for &target_slot in &operation.drag_slots {
                if target_slot == source_slot {
                    continue;
                }

                let split_amount = match operation.split_mode {
                    SplitMode::None => remaining_stack.count,
                    SplitMode::Even => (remaining_stack.count / operation.drag_slots.len() as u8).max(1),
                    SplitMode::Half => remaining_stack.count / 2,
                    SplitMode::Custom(amount) => amount.min(remaining_stack.count),
                };

                if let Some(mut split_stack) = remaining_stack.split(split_amount) {
                    if let Some(target_stack) = inventory.get_slot_mut(target_slot) {
                        if target_stack.is_none() || target_stack.as_ref().unwrap().is_empty() {
                            *target_stack = Some(split_stack);
                        } else if let Some(ref mut existing) = target_stack {
                            if existing.can_merge_with(&split_stack) {
                                existing.merge(&mut split_stack);
                            }
                        }
                    }
                }

                if remaining_stack.is_empty() {
                    break;
                }
            }

            // Put remaining items back in source slot
            if !remaining_stack.is_empty() {
                inventory.set_slot(source_slot, Some(remaining_stack));
            }
        }

        self.is_dragging = false;
        self.drag_start_slot = None;
        true
    }

    pub fn cancel_drag(&mut self) {
        self.current_operation = None;
        self.is_dragging = false;
        self.drag_start_slot = None;
    }

    pub fn get_drag_preview(&self, inventory: &Inventory) -> Vec<(usize, ItemStack)> {
        let mut preview = Vec::new();

        if let Some(ref operation) = self.current_operation {
            let total_slots = operation.drag_slots.len();
            
            for (i, &slot_index) in operation.drag_slots.iter().enumerate() {
                let split_amount = match operation.split_mode {
                    SplitMode::None => {
                        if i == 0 {
                            operation.source_stack.count
                        } else {
                            0
                        }
                    }
                    SplitMode::Even => (operation.source_stack.count / total_slots as u8).max(1),
                    SplitMode::Half => operation.source_stack.count / 2,
                    SplitMode::Custom(amount) => amount.min(operation.source_stack.count),
                };

                if split_amount > 0 {
                    preview.push((slot_index, ItemStack::new(operation.source_stack.item_type, split_amount)));
                }
            }
        }

        preview
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingGrid {
    pub slots: Vec<Option<ItemStack>>,
    pub width: usize,
    pub height: usize,
}

impl CraftingGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let total_slots = width * height;
        let mut slots = Vec::with_capacity(total_slots);
        
        for _ in 0..total_slots {
            slots.push(None);
        }

        Self {
            slots,
            width,
            height,
        }
    }

    pub fn get_slot(&self, x: usize, y: usize) -> Option<&ItemStack> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.slots[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_slot_mut(&mut self, x: usize, y: usize) -> Option<&mut Option<ItemStack>> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(&mut self.slots[index])
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, x: usize, y: usize, stack: Option<ItemStack>) -> bool {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.slots[index] = stack;
            true
        } else {
            false
        }
    }

    pub fn get_index_from_coords(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_coords_from_index(&self, index: usize) -> Option<(usize, usize)> {
        if index < self.slots.len() {
            Some((index % self.width, index / self.width))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    pub fn apply_drag_split(&mut self, drag_handler: &InventoryDragHandler) -> bool {
        if let Some(ref operation) = drag_handler.current_operation {
            let total_slots = operation.drag_slots.len();
            
            for (i, &slot_index) in operation.drag_slots.iter().enumerate() {
                if let Some((x, y)) = self.get_coords_from_index(slot_index) {
                    let split_amount = match operation.split_mode {
                        SplitMode::None => {
                            if i == 0 {
                                operation.source_stack.count
                            } else {
                                0
                            }
                        }
                        SplitMode::Even => (operation.source_stack.count / total_slots as u8).max(1),
                        SplitMode::Half => operation.source_stack.count / 2,
                        SplitMode::Custom(amount) => amount.min(operation.source_stack.count),
                    };

                    if split_amount > 0 {
                        let split_stack = ItemStack::new(operation.source_stack.item_type, split_amount);
                        self.set_slot(x, y, Some(split_stack));
                    }
                }
            }
            true
        } else {
            false
        }
    }
}
