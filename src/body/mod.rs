//! Body simulation. Milestone 2 per docs/REBUILD_PLAN.md: wounds feed a
//! cardiovascular model (currently just blood volume) that determines
//! consciousness and death. Infection and treatment are not implemented
//! yet; see the module docs on `state` for what `BodyState` tracks.

mod cardio;
mod integration;
mod region;
mod state;
mod wound;

pub use cardio::{Cardio, DEATH_THRESHOLD, KO_THRESHOLD};
pub use integration::{Body, BodyPlugin};
pub use region::BodyRegion;
pub use state::BodyState;
pub use wound::{landing_wound, Wound, WoundKind, FRACTURE_SEVERITY, LACERATION_SEVERITY};
