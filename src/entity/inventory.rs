use serde::{Deserialize, Serialize};

/// A stack of items in an inventory slot (can be materials or tools)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemStack {
    /// Stackable material with count
    Material { material_id: u16, count: u32 },
    /// Non-stackable tool with durability
    Tool { tool_id: u16, durability: u32 },
}

impl ItemStack {
    /// Create a new material stack
    pub fn new_material(material_id: u16, count: u32) -> Self {
        ItemStack::Material { material_id, count }
    }

    /// Create a new tool
    pub fn new_tool(tool_id: u16, durability: u32) -> Self {
        ItemStack::Tool {
            tool_id,
            durability,
        }
    }

    /// Get the material ID if this is a material stack
    pub fn material_id(&self) -> Option<u16> {
        match self {
            ItemStack::Material { material_id, .. } => Some(*material_id),
            ItemStack::Tool { .. } => None,
        }
    }

    /// Get the tool ID if this is a tool
    pub fn tool_id(&self) -> Option<u16> {
        match self {
            ItemStack::Tool { tool_id, .. } => Some(*tool_id),
            ItemStack::Material { .. } => None,
        }
    }

    /// Get the count (materials) or 1 (tools)
    pub fn count(&self) -> u32 {
        match self {
            ItemStack::Material { count, .. } => *count,
            ItemStack::Tool { .. } => 1, // Tools don't stack
        }
    }

    /// Get the maximum stack size
    pub fn max_stack_size(&self) -> u32 {
        match self {
            ItemStack::Material { .. } => 999, // Materials stack to 999
            ItemStack::Tool { .. } => 1,       // Tools don't stack
        }
    }

    /// Check if this stack can accept more items (only for materials)
    pub fn can_add(&self, amount: u32) -> bool {
        match self {
            ItemStack::Material { count, .. } => count + amount <= 999,
            ItemStack::Tool { .. } => false, // Tools never stack
        }
    }

    /// Add items to this stack (only works for materials), returns amount that didn't fit
    pub fn add(&mut self, amount: u32) -> u32 {
        match self {
            ItemStack::Material { count, .. } => {
                let space = 999u32.saturating_sub(*count);
                let to_add = amount.min(space);
                *count += to_add;
                amount - to_add
            }
            ItemStack::Tool { .. } => amount, // Can't add to tools
        }
    }

    /// Remove items from this stack, returns amount actually removed
    pub fn remove(&mut self, amount: u32) -> u32 {
        match self {
            ItemStack::Material { count, .. } => {
                let to_remove = amount.min(*count);
                *count -= to_remove;
                to_remove
            }
            ItemStack::Tool { .. } => {
                if amount > 0 {
                    1 // Removing a tool removes the whole thing
                } else {
                    0
                }
            }
        }
    }

    /// Check if this stack is empty
    pub fn is_empty(&self) -> bool {
        match self {
            ItemStack::Material { count, .. } => *count == 0,
            ItemStack::Tool { durability, .. } => *durability == 0, // Broken tools are "empty"
        }
    }

    /// Check if this stack is full
    pub fn is_full(&self) -> bool {
        match self {
            ItemStack::Material { count, .. } => *count >= 999,
            ItemStack::Tool { .. } => true, // Tools are always "full" (don't stack)
        }
    }

    /// Damage a tool's durability (returns true if tool broke)
    pub fn damage_tool(&mut self, damage: u32) -> bool {
        match self {
            ItemStack::Tool { durability, .. } => {
                *durability = durability.saturating_sub(damage);
                *durability == 0
            }
            ItemStack::Material { .. } => false,
        }
    }
}

/// Player inventory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub max_slots: usize,
}

impl Inventory {
    /// Create a new inventory with the specified number of slots
    pub fn new(max_slots: usize) -> Self {
        Inventory {
            slots: vec![None; max_slots],
            max_slots,
        }
    }

    /// Try to add a material to the inventory
    /// Returns the amount that couldn't be added (0 if all added successfully)
    pub fn add_item(&mut self, material_id: u16, mut amount: u32) -> u32 {
        // First, try to add to existing stacks of the same material
        for stack in self.slots.iter_mut().flatten() {
            if let ItemStack::Material {
                material_id: stack_id,
                ..
            } = stack
            {
                if *stack_id == material_id && !stack.is_full() {
                    amount = stack.add(amount);
                    if amount == 0 {
                        return 0;
                    }
                }
            }
        }

        // Then, try to create new stacks in empty slots
        while amount > 0 {
            match self.find_empty_slot() {
                Some(index) => {
                    let to_add = amount.min(999);
                    self.slots[index] = Some(ItemStack::new_material(material_id, to_add));
                    amount -= to_add;
                }
                None => break, // No empty slots, return remaining amount
            }
        }

        amount
    }

    /// Try to add a tool to the inventory
    /// Returns true if successful, false if no empty slot
    pub fn add_tool(&mut self, tool_id: u16, durability: u32) -> bool {
        match self.find_empty_slot() {
            Some(index) => {
                self.slots[index] = Some(ItemStack::new_tool(tool_id, durability));
                true
            }
            None => false, // No empty slots
        }
    }

    /// Try to remove a material from the inventory
    /// Returns the amount actually removed
    pub fn remove_item(&mut self, material_id: u16, mut amount: u32) -> u32 {
        let mut removed = 0;

        for slot in &mut self.slots {
            if let Some(stack) = slot {
                if let ItemStack::Material {
                    material_id: stack_id,
                    ..
                } = stack
                {
                    if *stack_id == material_id {
                        let to_remove = stack.remove(amount);
                        removed += to_remove;
                        amount -= to_remove;

                        // Remove empty stacks
                        if stack.is_empty() {
                            *slot = None;
                        }

                        if amount == 0 {
                            break;
                        }
                    }
                }
            }
        }

        removed
    }

    /// Remove a tool from the inventory by ID
    /// Returns true if found and removed
    pub fn remove_tool(&mut self, tool_id: u16) -> bool {
        for slot in &mut self.slots {
            if let Some(ItemStack::Tool {
                tool_id: slot_tool_id,
                ..
            }) = slot
            {
                if *slot_tool_id == tool_id {
                    *slot = None;
                    return true;
                }
            }
        }
        false
    }

    /// Check if the inventory contains at least the specified amount of a material
    pub fn has_item(&self, material_id: u16, amount: u32) -> bool {
        self.count_item(material_id) >= amount
    }

    /// Count how many of a specific material are in the inventory
    pub fn count_item(&self, material_id: u16) -> u32 {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter_map(|stack| match stack {
                ItemStack::Material {
                    material_id: stack_id,
                    count,
                } if *stack_id == material_id => Some(*count),
                _ => None,
            })
            .sum()
    }

    /// Find a tool in inventory and get its durability
    pub fn get_tool_durability(&self, tool_id: u16) -> Option<u32> {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .find_map(|stack| match stack {
                ItemStack::Tool {
                    tool_id: stack_tool_id,
                    durability,
                } if *stack_tool_id == tool_id => Some(*durability),
                _ => None,
            })
    }

    /// Damage a tool in the inventory (returns true if tool broke and was removed)
    pub fn damage_tool(&mut self, tool_id: u16, damage: u32) -> bool {
        for slot in &mut self.slots {
            if let Some(ItemStack::Tool {
                tool_id: slot_tool_id,
                ..
            }) = slot
            {
                if *slot_tool_id == tool_id {
                    let broke = slot.as_mut().unwrap().damage_tool(damage);
                    if broke {
                        *slot = None; // Remove broken tool
                        return true;
                    }
                    return false;
                }
            }
        }
        false
    }

    /// Find the first empty slot index
    fn find_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|slot| slot.is_none())
    }

    /// Get the number of empty slots
    pub fn empty_slot_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_none()).count()
    }

    /// Get the number of used slots
    pub fn used_slot_count(&self) -> usize {
        self.max_slots - self.empty_slot_count()
    }

    /// Clear all items from the inventory
    pub fn clear(&mut self) {
        self.slots.fill(None);
    }

    /// Get a reference to a slot
    pub fn get_slot(&self, index: usize) -> Option<&Option<ItemStack>> {
        self.slots.get(index)
    }

    /// Get a mutable reference to a slot
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Option<ItemStack>> {
        self.slots.get_mut(index)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new(50) // Default 50 slots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_basic() {
        let mut stack = ItemStack::new_material(1, 10);
        assert_eq!(stack.count(), 10);
        assert!(!stack.is_empty());
        assert!(!stack.is_full());

        stack.add(5);
        assert_eq!(stack.count(), 15);

        let removed = stack.remove(7);
        assert_eq!(removed, 7);
        assert_eq!(stack.count(), 8);
    }

    #[test]
    fn test_item_stack_overflow() {
        let mut stack = ItemStack::new_material(1, 990);
        let overflow = stack.add(20);
        assert_eq!(stack.count(), 999);
        assert_eq!(overflow, 11);
        assert!(stack.is_full());
    }

    #[test]
    fn test_tool_stack() {
        let mut tool = ItemStack::new_tool(1000, 50);
        assert_eq!(tool.tool_id(), Some(1000));
        assert_eq!(tool.count(), 1); // Tools don't stack
        assert!(tool.is_full()); // Tools are always "full"
        assert!(!tool.is_empty());

        // Damage tool
        let broke = tool.damage_tool(30);
        assert!(!broke);
        if let ItemStack::Tool { durability, .. } = tool {
            assert_eq!(durability, 20);
        }

        // Break tool
        let broke = tool.damage_tool(20);
        assert!(broke);
        assert!(tool.is_empty());
    }

    #[test]
    fn test_inventory_add_single() {
        let mut inv = Inventory::new(10);
        let remaining = inv.add_item(1, 50);
        assert_eq!(remaining, 0);
        assert_eq!(inv.count_item(1), 50);
        assert_eq!(inv.used_slot_count(), 1);
    }

    #[test]
    fn test_inventory_add_multiple_stacks() {
        let mut inv = Inventory::new(10);
        let remaining = inv.add_item(1, 2000);
        assert_eq!(remaining, 0);
        assert_eq!(inv.count_item(1), 2000);
        assert_eq!(inv.used_slot_count(), 3); // 999 + 999 + 2
    }

    #[test]
    fn test_inventory_add_to_existing() {
        let mut inv = Inventory::new(10);
        inv.add_item(1, 100);
        inv.add_item(1, 50);
        assert_eq!(inv.count_item(1), 150);
        assert_eq!(inv.used_slot_count(), 1); // Should stack together
    }

    #[test]
    fn test_inventory_remove() {
        let mut inv = Inventory::new(10);
        inv.add_item(1, 100);
        let removed = inv.remove_item(1, 30);
        assert_eq!(removed, 30);
        assert_eq!(inv.count_item(1), 70);
    }

    #[test]
    fn test_inventory_remove_multiple_stacks() {
        let mut inv = Inventory::new(10);
        inv.add_item(1, 1500); // Creates 2 stacks (999 + 501)
        let removed = inv.remove_item(1, 1200);
        assert_eq!(removed, 1200);
        assert_eq!(inv.count_item(1), 300);
        assert_eq!(inv.used_slot_count(), 1); // First stack removed
    }

    #[test]
    fn test_inventory_full() {
        let mut inv = Inventory::new(2);
        inv.add_item(1, 999);
        inv.add_item(2, 999);
        let remaining = inv.add_item(3, 100);
        assert_eq!(remaining, 100); // No space
        assert_eq!(inv.empty_slot_count(), 0);
    }

    #[test]
    fn test_inventory_has_item() {
        let mut inv = Inventory::new(10);
        inv.add_item(1, 100);
        assert!(inv.has_item(1, 50));
        assert!(inv.has_item(1, 100));
        assert!(!inv.has_item(1, 101));
        assert!(!inv.has_item(2, 1));
    }
}
