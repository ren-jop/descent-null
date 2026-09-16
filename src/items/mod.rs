//! items: pickups scattered through the cave, a weight-limited inventory
//! to carry them in, basic crafting, and using supplies to treat
//! injuries or eat/drink.

mod craft;
mod integration;
mod inventory;
mod item;

pub use craft::try_craft;
pub use integration::{ItemsPlugin, LastEvent, Pickup, PlayerInventory};
pub use inventory::Inventory;
pub use item::{ItemKind, ItemStack};
