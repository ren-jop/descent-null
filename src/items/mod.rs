//! items: pickups scattered through the cave, stack-based inventory,
//! deliberate crafting, and selected-item use.

mod craft;
mod integration;
mod inventory;
mod item;

pub use craft::{first_craftable, recipe_descriptions, try_craft};
pub use integration::{
    CraftingMenu, ItemsPlugin, LastEvent, Pickup, PlayerInventory, SelectedSlot,
};
pub use inventory::Inventory;
pub use item::{ItemKind, ItemStack};
