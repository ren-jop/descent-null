//! Bevy wiring for wounds, cardio and movement consequences.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::{CharacterController, LandingImpact};

use super::region::BodyRegion;
use super::state::BodyState;
use super::wound::landing_wound;

const LANDING_REGIONS: [BodyRegion; 2] = [BodyRegion::LeftLeg, BodyRegion::RightLeg];
const FRACTURE_SPEED_CAP: f32 = 60.0;
/// Dropping several cave layers should not be a shortcut to the objective.
/// Above this impact speed the landing is treated as unsurvivable trauma.
const CATASTROPHIC_LANDING_SPEED: f32 = 1150.0;

#[derive(Component, Default)]
pub struct Body(pub BodyState);

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_landing_wounds, tick_cardio, enforce_unconsciousness, enforce_fracture_limp).chain(),
        );
    }
}

fn apply_landing_wounds(mut impacts: MessageReader<LandingImpact>, mut bodies: Query<&mut Body>) {
    for impact in impacts.read() {
        let Ok(mut body) = bodies.get_mut(impact.entity) else {
            continue;
        };
        if impact.downward_speed >= CATASTROPHIC_LANDING_SPEED {
            body.0.apply_external_drain(1.0);
            continue;
        }
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

fn enforce_fracture_limp(mut query: Query<(&Body, &mut LinearVelocity), With<CharacterController>>) {
    for (body, mut velocity) in &mut query {
        if body.0.has_untreated_leg_fracture() {
            velocity.x = velocity.x.clamp(-FRACTURE_SPEED_CAP, FRACTURE_SPEED_CAP);
        }
    }
}
