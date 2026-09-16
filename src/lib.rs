//! Descent: Null — systemic 2D survival simulation core.
//! Phase 1 (per the survival-loop design doc): physics, procedural cave,
//! body/cardio, survival (hunger/thirst/stamina), and items/inventory are
//! all in. Phase 2 (crafting, hazards, enemies, light) is next.

pub mod body;
pub mod enemy;
pub mod game;
pub mod items;
pub mod physics;
pub mod player;
pub mod survival;
pub mod ui;
pub mod world;
