//! simple crafting: fixed recipes, first one you can afford gets made.
//! no crafting menu yet — pressing the craft key just tries them in
//! order. pure logic, no bevy.

use super::inventory::Inventory;
use super::item::{ItemKind, ItemStack};

struct Recipe {
    inputs: &'static [(ItemKind, u32)],
    output: ItemStack,
    purpose: &'static str,
}

const RECIPES: &[Recipe] = &[
    Recipe {
        inputs: &[(ItemKind::Scrap, 3), (ItemKind::Cloth, 1)],
        output: ItemStack { kind: ItemKind::Bandage, quantity: 1 },
        purpose: "stops active bleeding",
    },
    Recipe {
        inputs: &[(ItemKind::Metal, 2), (ItemKind::Scrap, 1)],
        output: ItemStack { kind: ItemKind::Splint, quantity: 1 },
        purpose: "stabilises a fracture and restores movement",
    },
    Recipe {
        inputs: &[(ItemKind::Cloth, 1), (ItemKind::Metal, 1), (ItemKind::Battery, 1)],
        output: ItemStack { kind: ItemKind::Medkit, quantity: 1 },
        purpose: "restores blood volume and treats pain",
    },
];

/// Human-readable recipe list for the HUD. Every line states both the
/// ingredients and the gameplay reason to make the item, so crafting is
/// a decision rather than a memory test.
pub fn recipe_descriptions() -> Vec<String> {
    RECIPES
        .iter()
        .map(|recipe| {
            let inputs = recipe
                .inputs
                .iter()
                .map(|(kind, qty)| format!("{qty} {}", kind.label()))
                .collect::<Vec<_>>()
                .join(" + ");
            format!(
                "{inputs} -> {}  •  {}",
                recipe.output.kind.label(),
                recipe.purpose
            )
        })
        .collect()
}

/// tries each recipe in order; crafts (consumes inputs, adds output) the
/// first one the inventory can afford. returns the crafted kind, if any.
pub fn try_craft(inventory: &mut Inventory) -> Option<ItemKind> {
    let recipe = RECIPES
        .iter()
        .find(|r| r.inputs.iter().all(|(kind, qty)| inventory.quantity_of(*kind) >= *qty))?;
    for (kind, qty) in recipe.inputs {
        inventory.remove(*kind, *qty);
    }
    inventory.add(recipe.output);
    Some(recipe.output.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafts_bandage_when_materials_available() {
        let mut inv = Inventory::new(20);
        inv.add(ItemStack::new(ItemKind::Scrap, 3));
        inv.add(ItemStack::new(ItemKind::Cloth, 1));
        assert_eq!(try_craft(&mut inv), Some(ItemKind::Bandage));
        assert_eq!(inv.quantity_of(ItemKind::Bandage), 1);
        assert_eq!(inv.quantity_of(ItemKind::Scrap), 0);
        assert_eq!(inv.quantity_of(ItemKind::Cloth), 0);
    }

    #[test]
    fn does_nothing_without_materials() {
        let mut inv = Inventory::new(20);
        assert_eq!(try_craft(&mut inv), None);
    }

    #[test]
    fn skips_unaffordable_recipes_for_an_affordable_one() {
        let mut inv = Inventory::new(20);
        // not enough for bandage (needs 3 scrap), but enough for a splint.
        inv.add(ItemStack::new(ItemKind::Scrap, 1));
        inv.add(ItemStack::new(ItemKind::Metal, 2));
        assert_eq!(try_craft(&mut inv), Some(ItemKind::Splint));
    }

    #[test]
    fn recipe_descriptions_lists_every_recipe_and_why_it_matters() {
        let descriptions = recipe_descriptions();
        assert_eq!(descriptions.len(), RECIPES.len());
        assert!(descriptions.iter().any(|d| d.contains("bandage") && d.contains("bleeding")));
        assert!(descriptions.iter().any(|d| d.contains("splint") && d.contains("fracture")));
        assert!(descriptions.iter().any(|d| d.contains("medkit") && d.contains("blood")));
    }
}
