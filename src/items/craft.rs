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
        purpose: "Stops bleeding and restores some health",
    },
    Recipe {
        inputs: &[(ItemKind::Metal, 2), (ItemKind::Scrap, 1)],
        output: ItemStack { kind: ItemKind::Splint, quantity: 1 },
        purpose: "Stabilises a leg fracture",
    },
    Recipe {
        inputs: &[(ItemKind::Cloth, 1), (ItemKind::Metal, 1), (ItemKind::Battery, 1)],
        output: ItemStack { kind: ItemKind::Medkit, quantity: 1 },
        purpose: "Restores more health and treats pain",
    },
];

pub fn recipe_count() -> usize {
    RECIPES.len()
}

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

/// Returns the first currently craftable result. Used only for a compact HUD
/// hint; the player still chooses a recipe in the crafting menu.
pub fn first_craftable(inventory: &Inventory) -> Option<ItemKind> {
    RECIPES
        .iter()
        .enumerate()
        .find(|(index, _)| can_craft(inventory, *index))
        .map(|(_, recipe)| recipe.output.kind)
}

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
    inventory.add(recipe.output);
    Some(recipe.output.kind)
}

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
    fn craft_ready_reports_available_output() {
        let mut inv = Inventory::new(20);
        inv.add(ItemStack::new(ItemKind::Scrap, 3));
        inv.add(ItemStack::new(ItemKind::Cloth, 1));
        assert_eq!(first_craftable(&inv), Some(ItemKind::Bandage));
    }

    #[test]
    fn descriptions_are_numbered_and_ascii_friendly() {
        let descriptions = recipe_descriptions();
        assert_eq!(descriptions.len(), recipe_count());
        assert!(descriptions[0].contains("[1] BANDAGE"));
    }
}
