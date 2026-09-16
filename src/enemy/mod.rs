//! Small readable enemy roster: crawlers apply steady pressure while faster
//! skitters make deeper ledges more dangerous. Both share the same simple
//! idle/chase/attack state model.

mod integration;
mod state;

pub use integration::{spawn_enemy, spawn_skitter, Enemy, EnemyKind, EnemyPlugin};
pub use state::{EnemyState, EnemyStats};
