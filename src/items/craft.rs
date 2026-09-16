//! Small recipe catalogue used by the crafting menu.
//! Crafting is deliberate: the player chooses a numbered recipe instead
//! of `C` silently crafting the first affordable result.

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
        purpose: "Stops active bleeding",
    },
    Recipe {
        inputs: &[(ItemKind::Metal, 2), (ItemKind::Scrap, 1)],
        output: ItemStack { kind: ItemKind::Splint, quantity: 1 },
        purpose: "Stabilises a leg fracture",
    },
    Recipe {
        inputs: &[(ItemKind::Cloth, 1), (ItemKind::Metal, 1), (ItemKind::Battery, 1)],
        output: ItemStack { kind: ItemKind::Medkit, quantity: 1 },
        purpose: "Restores blood and treats pain",
    },
];

pub fn recipe_count() -> usize {
    RECIPES.len()
}

/// ASCII-only descriptions: the default bundled font used on some Macs
/// does not contain the old Unicode bullet glyph, which rendered as boxes.
pub fn recipe_descriptions() -> Vec<String> {
    RECIPES
        .iter()
        .enumerate()
        .map(|(index, recipe)| {
            let inputs = recipe
                .inputs
                .iter()
                .map(|(kind, qty)| format!("{qty} {}", kind.label()))
                .collect::<Vec<_>>()
                .join(" + ");
            format!(
                "[{}] {}\n    Need: {}\n    Use: {}",
                index + 1,
                recipe.output.kind.label().to_uppercase(),
                inputs,
                recipe.purpose,
            )
        })
        .collect()
}

pub fn can_craft(inventory: &Inventory, index: usize) -> bool {
    RECIPES
        .get(index)
        .map(|recipe| {
            recipe
                .inputs
                .iter()
                .all(|(kind, qty)| inventory.quantity_of(*kind) >= *qty)
        })
        .unwrap_or(false)
}

/// Craft exactly the recipe the player selected. Returns the output kind
/// when successful; `None` means the recipe does not exist or ingredients
/// are missing.
pub fn try_craft_index(inventory: &mut Inventory, index: usize) -> Option<ItemKind> {
    let recipe = RECIPES.get(index)?;
    if !recipe
        .inputs
        .iter()
        .all(|(kind, qty)| inventory.quantity_of(*kind) >= *qty)
    {
        return None;
    }
    for (kind, qty) in recipe.inputs {
        inventory.remove(*kind, *qty);
    }
    if inventory.add(recipe.output) {
        Some(recipe.output.kind)
    } else {
        // Inputs normally reduce weight, so this should be rare. Keeping
        // the return type simple is fine for the current tiny catalogue.
        None
    }
}

/// Kept for tests / compatibility. The actual game UI now calls
/// `try_craft_index` after the player chooses a numbered recipe.
pub fn try_craft(inventory: &mut Inventory) -> Option<ItemKind> {
    let index = (0..RECIPES.len()).find(|index| can_craft(inventory, *index))?;
    try_craft_index(inventory, index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafts_selected_bandage_when_materials_available() {
        let mut inv = Inventory::new(20);
        inv.add(ItemStack::new(ItemKind::Scrap, 3));
        inv.add(ItemStack::new(ItemKind::Cloth, 1));
        assert_eq!(try_craft_index(&mut inv, 0), Some(ItemKind::Bandage));
        assert_eq!(inv.quantity_of(ItemKind::Bandage), 1);
    }

    #[test]
    fn refuses_selected_recipe_without_materials() {
        let mut inv = Inventory::new(20);
        assert_eq!(try_craft_index(&mut inv, 2), None);
    }

    #[test]
    fn descriptions_are_numbered_and_ascii_friendly() {
        let descriptions = recipe_descriptions();
        assert_eq!(descriptions.len(), recipe_count());
        assert!(descriptions[0].contains("[1] BANDAGE"));
        assert!(descriptions[0].contains("Stops active bleeding"));
    }
}
