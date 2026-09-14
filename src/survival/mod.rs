//! Survival meters. First slice of milestone 3 per docs/REBUILD_PLAN.md:
//! hunger and thirst only, draining over real time and — once empty —
//! eroding the same blood-volume/consciousness pipeline `body` uses for
//! wounds. No food/water items to refill them yet, no temperature, no
//! fatigue/stamina.

mod integration;
mod state;

pub use integration::{Survival, SurvivalPlugin};
pub use state::SurvivalState;
