//! one simple, reliable enemy — a cave crawler with idle/chase/attack
//! behavior. spawned as part of world generation (world::cave), not a
//! separate showcase. see integration.rs for why it's a flat 3-state
//! machine instead of a behavior-tree framework: that's deliberate.

mod integration;
mod state;

pub use integration::{spawn_enemy, Enemy, EnemyPlugin};
pub use state::{EnemyState, EnemyStats};
