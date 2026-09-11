//! Character physics: acceleration, jump assist, landing impacts.
//! The numeric helpers here are engine-agnostic so they can be unit-tested
//! without spinning up Bevy.

mod controller;
mod jump;
mod landing;

pub use controller::{
    CharacterController, CharacterControllerBundle, Grounded, LandingImpact,
    PhysicsGameplayPlugin,
};
pub use jump::{JumpAssist, JUMP_BUFFER, JUMP_COYOTE};
pub use landing::{landing_severity, FallTracker, LANDING_SPEED_THRESHOLD};
