//! Stack-based inventory. Pure data, no Bevy.
//!
//! Items of the same kind always merge into one stack. `capacity` is kept as
//! legacy metadata for older callers/tests, but it is no longer a hard pickup
//! limit: the backpack can grow naturally and the UI reports carried weight.

use super::item::{ItemKind, ItemStack};

#[derive(Clone, Debug, PartialEq)]
pub struct Inventory {
    stacks: Vec<ItemStack>,
    capacity: u32,
}

impl Inventory {
    pub fn new(capacity: u32) -> Self {
        Self { stacks: Vec::new(), capacity }
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn used_weight(&self) -> u32 {
        self.stacks.iter().map(ItemStack::total_weight).sum()
    }

    pub fn stacks(&self) -> &[ItemStack] {
        &self.stacks
    }

    pub fn quantity_of(&self, kind: ItemKind) -> u32 {
        self.stacks
            .iter()
            .filter(|stack| stack.kind == kind)
            .map(|stack| stack.quantity)
            .sum()
    }

    /// Adds a stack and merges it with an existing stack of the same item.
    /// There is intentionally no per-stack or hard backpack cap.
    pub fn add(&mut self, incoming: ItemStack) -> bool {
        if let Some(existing) = self
            .stacks
            .iter_mut()
            .find(|stack| stack.kind == incoming.kind)
        {
            existing.quantity = existing.quantity.saturating_add(incoming.quantity);
        } else {
            self.stacks.push(incoming);
        }
        true
    }

    /// Removes up to `quantity` of `kind`. Returns how much was removed.
    pub fn remove(&mut self, kind: ItemKind, quantity: u32) -> u32 {
        let Some(index) = self.stacks.iter().position(|stack| stack.kind == kind) else {
            return 0;
        };
        let removed = quantity.min(self.stacks[index].quantity);
        self.stacks[index].quantity -= removed;
        if self.stacks[index].quantity == 0 {
            self.stacks.remove(index);
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_items_succeeds_even_past_legacy_capacity() {
        let mut inv = Inventory::new(2);
        assert!(inv.add(ItemStack::new(ItemKind::Metal, 30)));
        assert_eq!(inv.quantity_of(ItemKind::Metal), 30);
        assert_eq!(inv.stacks().len(), 1);
    }

    #[test]
    fn same_kind_merges_into_one_unlimited_stack() {
        let mut inv = Inventory::new(10);
        inv.add(ItemStack::new(ItemKind::Water, 1));
        inv.add(ItemStack::new(ItemKind::Water, 200));
        assert_eq!(inv.stacks().len(), 1);
        assert_eq!(inv.quantity_of(ItemKind::Water), 201);
    }

    #[test]
    fn remove_takes_at_most_what_exists() {
        let mut inv = Inventory::new(10);
        inv.add(ItemStack::new(ItemKind::Bandage, 2));
        assert_eq!(inv.remove(ItemKind::Bandage, 5), 2);
        assert_eq!(inv.quantity_of(ItemKind::Bandage), 0);
        assert_eq!(inv.remove(ItemKind::Bandage, 1), 0);
    }

    #[test]
    fn emptied_stack_is_removed_from_the_list() {
        let mut inv = Inventory::new(10);
        inv.add(ItemStack::new(ItemKind::Food, 1));
        inv.remove(ItemKind::Food, 1);
        assert!(inv.stacks().is_empty());
    }
}
