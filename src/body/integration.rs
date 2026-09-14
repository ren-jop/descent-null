//! Bevy wiring: wraps `BodyState` as a component and turns landing-impact
//! messages into wounds. Kept separate from `state.rs`/`wound.rs` so those
//! stay testable without spinning up an `App` (same split as
//! `physics::controller` vs `physics::jump`/`physics::landing`).

use bevy::prelude::*;

use crate::physics::LandingImpact;

use super::region::BodyRegion;
use super::state::BodyState;
use super::wound::landing_wound;

/// Regions that take the hit on a hard landing. A real per-limb resolver
/// driven by landing pose is future work — see the milestone 2 entry in
/// docs/REBUILD_PLAN.md. For now a hard landing lands on both legs.
const LANDING_REGIONS: [BodyRegion; 2] = [BodyRegion::LeftLeg, BodyRegion::RightLeg];

#[derive(Component, Default)]
pub struct Body(pub BodyState);

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_landing_wounds);
    }
}

fn apply_landing_wounds(mut impacts: MessageReader<LandingImpact>, mut bodies: Query<&mut Body>) {
    for impact in impacts.read() {
        let Ok(mut body) = bodies.get_mut(impact.entity) else {
            continue;
        };
        for region in LANDING_REGIONS {
            if let Some(wound) = landing_wound(region, impact.severity) {
                body.0.apply_wound(wound);
            }
        }
    }
}
