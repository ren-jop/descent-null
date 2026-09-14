//! Bevy wiring: wraps `BodyState` as a component, turns landing-impact
//! messages into wounds, ticks the cardiovascular model, and locks out
//! movement while unconscious. Kept separate from `state.rs`/`wound.rs`/
//! `cardio.rs` so those stay testable without spinning up an `App` (same
//! split as `physics::controller` vs `physics::jump`/`physics::landing`).

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::{CharacterController, LandingImpact};

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
        app.add_systems(
            Update,
            (apply_landing_wounds, tick_cardio, enforce_unconsciousness).chain(),
        );
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

fn tick_cardio(time: Res<Time>, mut bodies: Query<&mut Body>) {
    let dt = time.delta_secs();
    for mut body in &mut bodies {
        body.0.tick(dt);
    }
}

/// While unconscious (or dead), the body stops responding to input: no
/// horizontal drift, no jump impulse. Gravity/collision still apply, so an
/// unconscious character still falls and settles — it just can't act.
fn enforce_unconsciousness(
    mut query: Query<(&Body, &mut LinearVelocity), With<CharacterController>>,
) {
    for (body, mut velocity) in &mut query {
        if body.0.is_ko() {
            velocity.x = 0.0;
            if velocity.y > 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}
