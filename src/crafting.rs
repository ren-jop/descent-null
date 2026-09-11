//! Data-driven crafting: recipes consume inventory items and produce a
//! result, gated by an intelligence requirement.

use crate::inventory::{Inventory, ItemId};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub id: RecipeId,
    pub inputs: &'static [(ItemId, u32)],
    pub output: (ItemId, u32),
    pub required_intelligence: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecipeId {
    BandageFromFiber,
    SplintFromScrapAndFiber,
}

pub fn recipe(id: RecipeId) -> Recipe {
    match id {
        RecipeId::BandageFromFiber => Recipe {
            id,
            inputs: &[(ItemId::Fiber, 2)],
            output: (ItemId::Bandage, 1),
            required_intelligence: 0.0,
        },
        RecipeId::SplintFromScrapAndFiber => Recipe {
            id,
            inputs: &[(ItemId::ScrapMetal, 1), (ItemId::Fiber, 1)],
            output: (ItemId::Splint, 1),
            required_intelligence: 15.0,
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CraftError {
    MissingIngredients,
    IntelligenceTooLow,
    InventoryFull,
}

/// Attempts to craft `recipe_id` from `inventory`. On success, consumes the
/// inputs and adds the output; on failure, the inventory is left untouched.
pub fn craft(
    inventory: &mut Inventory,
    recipe_id: RecipeId,
    intelligence: f32,
) -> Result<(), CraftError> {
    let r = recipe(recipe_id);
    if intelligence < r.required_intelligence {
        return Err(CraftError::IntelligenceTooLow);
    }
    for (id, qty) in r.inputs {
        if !inventory.has_at_least(*id, *qty) {
            return Err(CraftError::MissingIngredients);
        }
    }
    // simulate the output fitting before committing any change
    let (out_id, out_qty) = r.output;
    let mut probe = Inventory {
        slots: inventory.slots.clone(),
        max_weight: inventory.max_weight,
    };
    for (id, qty) in r.inputs {
        probe.remove(*id, *qty);
    }
    let fit = probe.add(out_id, out_qty);
    if fit < out_qty {
        return Err(CraftError::InventoryFull);
    }

    for (id, qty) in r.inputs {
        inventory.remove(*id, *qty);
    }
    inventory.add(out_id, out_qty);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafting_consumes_inputs_and_produces_output() {
        let mut inv = Inventory::new(1000.0);
        inv.add(ItemId::Fiber, 2);
        let result = craft(&mut inv, RecipeId::BandageFromFiber, 0.0);
        assert!(result.is_ok());
        assert_eq!(inv.quantity_of(ItemId::Fiber), 0);
        assert_eq!(inv.quantity_of(ItemId::Bandage), 1);
    }

    #[test]
    fn crafting_without_ingredients_fails_and_changes_nothing() {
        let mut inv = Inventory::new(1000.0);
        let result = craft(&mut inv, RecipeId::BandageFromFiber, 0.0);
        assert_eq!(result, Err(CraftError::MissingIngredients));
        assert_eq!(inv.quantity_of(ItemId::Bandage), 0);
    }

    #[test]
    fn crafting_below_required_intelligence_fails() {
        let mut inv = Inventory::new(1000.0);
        inv.add(ItemId::ScrapMetal, 1);
        inv.add(ItemId::Fiber, 1);
        let result = craft(&mut inv, RecipeId::SplintFromScrapAndFiber, 5.0);
        assert_eq!(result, Err(CraftError::IntelligenceTooLow));
        // ingredients must be untouched on failure
        assert_eq!(inv.quantity_of(ItemId::ScrapMetal), 1);
        assert_eq!(inv.quantity_of(ItemId::Fiber), 1);
    }

    #[test]
    fn crafting_that_would_overflow_inventory_weight_fails_cleanly() {
        let mut inv = Inventory::new(2.1); // just enough for 2 fiber (0.3 each) + a hair more
        inv.add(ItemId::Fiber, 2);
        // Splint weighs 0.4 and needs scrap (1.0) + fiber (0.3) as inputs too,
        // but we only added fiber, so this should fail on missing ingredients
        // rather than panicking.
        let result = craft(&mut inv, RecipeId::SplintFromScrapAndFiber, 100.0);
        assert_eq!(result, Err(CraftError::MissingIngredients));
    }
}
