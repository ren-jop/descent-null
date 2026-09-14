//! Descent: Null — systemic 2D survival simulation core.
//! Milestone 1 (player + physics) is done; `body` (wounds + cardio) and
//! `survival` (hunger/thirst) cover milestone 2's core loop and the first
//! slice of milestone 3. Later systems land as modules, not grid hacks.

pub mod body;
pub mod game;
pub mod physics;
pub mod player;
pub mod survival;
pub mod ui;
pub mod world;
