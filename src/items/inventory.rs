//! carry-weight-limited inventory. pure data, no bevy.

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
        self.stacks.iter().filter(|s| s.kind == kind).map(|s| s.quantity).sum()
    }

    /// tries to add the stack, merging into an existing stack of the same
    /// kind if there is one. returns false (and adds nothing) if it would
    /// go over capacity.
    pub fn add(&mut self, incoming: ItemStack) -> bool {
        if self.used_weight() + incoming.total_weight() > self.capacity {
            return false;
        }
        if let Some(existing) = self.stacks.iter_mut().find(|s| s.kind == incoming.kind) {
            existing.quantity += incoming.quantity;
        } else {
            self.stacks.push(incoming);
        }
        true
    }

    /// removes up to `quantity` of `kind`. returns how much was actually
    /// removed (may be less than requested, or zero).
    pub fn remove(&mut self, kind: ItemKind, quantity: u32) -> u32 {
        let Some(index) = self.stacks.iter().position(|s| s.kind == kind) else {
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
    fn adding_within_capacity_succeeds() {
        let mut inv = Inventory::new(10);
        assert!(inv.add(ItemStack::new(ItemKind::Scrap, 3)));
        assert_eq!(inv.quantity_of(ItemKind::Scrap), 3);
    }

    #[test]
    fn adding_past_capacity_fails_and_changes_nothing() {
        let mut inv = Inventory::new(2);
        assert!(!inv.add(ItemStack::new(ItemKind::Metal, 3)));
        assert_eq!(inv.quantity_of(ItemKind::Metal), 0);
        assert_eq!(inv.used_weight(), 0);
    }

    #[test]
    fn same_kind_merges_into_one_stack() {
        let mut inv = Inventory::new(10);
        inv.add(ItemStack::new(ItemKind::Water, 1));
        inv.add(ItemStack::new(ItemKind::Water, 2));
        assert_eq!(inv.stacks().len(), 1);
        assert_eq!(inv.quantity_of(ItemKind::Water), 3);
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
