//! Body simulation. Wounds feed a cardiovascular model that determines
//! consciousness and death; the integration layer also remembers the most
//! recent damage source so UI/audio can explain death without guessing.

mod cardio;
mod integration;
mod region;
mod state;
mod wound;

pub use cardio::{Cardio, DEATH_THRESHOLD, KO_THRESHOLD};
pub use integration::{Body, BodyPlugin, DamageCause, LastDamageCause};
pub use region::BodyRegion;
pub use state::BodyState;
pub use wound::{landing_wound, Wound, WoundKind, FRACTURE_SEVERITY, LACERATION_SEVERITY};
