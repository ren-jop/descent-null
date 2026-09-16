//! the game world: a procedurally generated, depth-layered cave.

mod cave;
mod records;

pub use cave::{CavePlugin, CurrentDepth, DepthAnnouncement, RunStats};
pub use records::{BestTimes, RecordsPlugin};
