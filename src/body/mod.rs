//! Body simulation. Milestone 2 per docs/REBUILD_PLAN.md — this is the
//! first slice only: landing impacts wound the legs with pain and (above
//! laceration severity) bleeding. Cardiovascular state, infection, and
//! treatment are not implemented yet; see the module docs on `state` for
//! what `BodyState` currently tracks.

mod integration;
mod region;
mod state;
mod wound;

pub use integration::{Body, BodyPlugin};
pub use region::BodyRegion;
pub use state::BodyState;
pub use wound::{landing_wound, Wound, WoundKind, FRACTURE_SEVERITY, LACERATION_SEVERITY};
