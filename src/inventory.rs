//! Weight-and-slot-constrained inventory. Items are data-driven via
//! `ItemDef`, not hard-coded gameplay special-cases.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ItemCategory {
    Food,
    Liquid,
    Medical,
    Material,
    Tool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ItemId {
    Ration,
    PurifiedWater,
    Bandage,
    Splint,
    Antiseptic,
    ClottingAgent,
    Painkiller,
    ScrapMetal,
    Fiber,
    Bracket, // a simple craftable tool part
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub id: ItemId,
    pub name: &'static str,
    pub weight: f32,
    pub stack_size: u32,
    pub category: ItemCategory,
}

pub fn item_def(id: ItemId) -> ItemDef {
    match id {
        ItemId::Ration => ItemDef { id, name: "ration pack", weight: 0.5, stack_size: 6, category: ItemCategory::Food },
        ItemId::PurifiedWater => ItemDef { id, name: "purified water", weight: 0.6, stack_size: 4, category: ItemCategory::Liquid },
        ItemId::Bandage => ItemDef { id, name: "bandage", weight: 0.1, stack_size: 10, category: ItemCategory::Medical },
        ItemId::Splint => ItemDef { id, name: "splint", weight: 0.4, stack_size: 4, category: ItemCategory::Medical },
        ItemId::Antiseptic => ItemDef { id, name: "antiseptic", weight: 0.2, stack_size: 5, category: ItemCategory::Medical },
        ItemId::ClottingAgent => ItemDef { id, name: "clotting agent", weight: 0.2, stack_size: 5, category: ItemCategory::Medical },
        ItemId::Painkiller => ItemDef { id, name: "painkiller", weight: 0.1, stack_size: 8, category: ItemCategory::Medical },
        ItemId::ScrapMetal => ItemDef { id, name: "scrap metal", weight: 1.0, stack_size: 10, category: ItemCategory::Material },
        ItemId::Fiber => ItemDef { id, name: "fiber bundle", weight: 0.3, stack_size: 10, category: ItemCategory::Material },
        ItemId::Bracket => ItemDef { id, name: "makeshift bracket", weight: 0.6, stack_size: 5, category: ItemCategory::Tool },
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub id: ItemId,
    pub quantity: u32,
}

pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub max_weight: f32,
}

impl Inventory {
    pub fn new(max_weight: f32) -> Self {
        Inventory { slots: Vec::new(), max_weight }
    }

    pub fn total_weight(&self) -> f32 {
        self.slots
            .iter()
            .map(|s| item_def(s.id).weight * s.quantity as f32)
            .sum()
    }

    pub fn quantity_of(&self, id: ItemId) -> u32 {
        self.slots.iter().filter(|s| s.id == id).map(|s| s.quantity).sum()
    }

    /// Attempts to add `qty` of `id`, respecting stack sizes and total
    /// weight capacity. Returns how many units actually fit.
    pub fn add(&mut self, id: ItemId, qty: u32) -> u32 {
        let def = item_def(id);
        let mut remaining = qty;
        let mut weight_used = self.total_weight();

        // top up existing partial stacks first
        for stack in self.slots.iter_mut().filter(|s| s.id == id) {
            if remaining == 0 {
                break;
            }
            let room = def.stack_size.saturating_sub(stack.quantity);
            let can_add = room.min(remaining);
            if can_add > 0 {
                let weight_after = weight_used + def.weight * can_add as f32;
                let allowed = if weight_after <= self.max_weight {
                    can_add
                } else {
                    let allowed_weight = self.max_weight - weight_used;
                    ((allowed_weight / def.weight).floor().max(0.0)) as u32
                };
                stack.quantity += allowed;
                weight_used += def.weight * allowed as f32;
                remaining -= allowed;
                if allowed < can_add {
                    return qty - remaining; // hit weight cap
                }
            }
        }

        // then new stacks
        while remaining > 0 {
            let weight_after_one = weight_used + def.weight;
            if weight_after_one > self.max_weight {
                break;
            }
            let mut take = remaining.min(def.stack_size);
            while take > 0 && weight_used + def.weight * take as f32 > self.max_weight {
                take -= 1;
            }
            if take == 0 {
                break;
            }
            self.slots.push(ItemStack { id, quantity: take });
            weight_used += def.weight * take as f32;
            remaining -= take;
        }

        qty - remaining
    }

    /// Removes up to `qty` of `id`. Returns how many were actually removed.
    pub fn remove(&mut self, id: ItemId, qty: u32) -> u32 {
        let mut remaining = qty;
        self.slots.retain_mut(|stack| {
            if stack.id != id || remaining == 0 {
                return true;
            }
            let take = stack.quantity.min(remaining);
            stack.quantity -= take;
            remaining -= take;
            stack.quantity > 0
        });
        qty - remaining
    }

    pub fn has_at_least(&self, id: ItemId, qty: u32) -> bool {
        self.quantity_of(id) >= qty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_items_respects_weight_capacity() {
        let mut inv = Inventory::new(2.0); // ScrapMetal weighs 1.0/unit
        let added = inv.add(ItemId::ScrapMetal, 5);
        assert_eq!(added, 2, "should only fit 2 units of a 1.0-weight item into a 2.0 cap");
        assert!(inv.total_weight() <= 2.0);
    }

    #[test]
    fn adding_items_respects_stack_size() {
        let mut inv = Inventory::new(1000.0);
        let added = inv.add(ItemId::Bandage, 25); // stack size 10
        assert_eq!(added, 25);
        assert_eq!(inv.slots.len(), 3); // 10 + 10 + 5
    }

    #[test]
    fn remove_takes_from_stacks_and_reports_actual_amount_removed() {
        let mut inv = Inventory::new(1000.0);
        inv.add(ItemId::Ration, 4);
        let removed = inv.remove(ItemId::Ration, 10);
        assert_eq!(removed, 4);
        assert_eq!(inv.quantity_of(ItemId::Ration), 0);
    }

    #[test]
    fn has_at_least_reflects_true_totals_across_stacks() {
        let mut inv = Inventory::new(1000.0);
        inv.add(ItemId::Fiber, 10);
        inv.add(ItemId::Fiber, 10);
        assert!(inv.has_at_least(ItemId::Fiber, 15));
        assert!(!inv.has_at_least(ItemId::Fiber, 25));
    }
}
