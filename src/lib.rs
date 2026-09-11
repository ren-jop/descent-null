//! Descent: Null -- an original 2D survival-exploration simulation core.
//! This library crate contains zero rendering/input code; see `src/main.rs`
//! (graphical) and `src/bin/tui.rs` (terminal) for front ends that drive it.

pub mod body;
pub mod crafting;
pub mod entities;
pub mod game;
pub mod inventory;
pub mod player;
pub mod survival;
pub mod world;
