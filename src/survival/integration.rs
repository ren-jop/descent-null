//! Bevy wiring for hunger and thirst. The model is intentionally direct:
//! low needs reduce movement, while critical dehydration starts reducing
//! blood volume so the health consequence is visible and understandable.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::physics::CharacterController;

use super::state::SurvivalState;

const NORMAL_SPEED_CAP: f32 = 180.0;
const LOW_NEEDS_MIN_SPEED_CAP: f32 = 105.0;
const CRITICAL_THIRST: f32 = 0.12;
/// At zero thirst this removes roughly 1.5% blood volume per second.
const DEHYDRATION_BLOOD_DRAIN_PER_SEC: f32 = 0.015;

#[derive(Component, Default)]
pub struct Survival(pub SurvivalState);

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tick_survival, apply_survival_consequences).chain());
    }
}

fn tick_survival(time: Res<Time>, mut query: Query<&mut Survival>) {
    let dt = time.delta_secs();
    for mut survival in &mut query {
        survival.0.tick(dt);
    }
}

fn apply_survival_consequences(
    time: Res<Time>,
    mut query: Query<(&Survival, &mut Body, &mut LinearVelocity), With<CharacterController>>,
) {
    let dt = time.delta_secs();
    for (survival, mut body, mut velocity) in &mut query {
        let hardship = survival
            .0
            .hunger_hardship()
            .max(survival.0.thirst_hardship());

        if hardship > 0.0 {
            let cap = NORMAL_SPEED_CAP
                - (NORMAL_SPEED_CAP - LOW_NEEDS_MIN_SPEED_CAP) * hardship;
            velocity.x = velocity.x.clamp(-cap, cap);
        }

        // Dehydration is the survival need with the clearest direct health
        // consequence. The drain ramps in only below 12%, avoiding mysterious
        // health loss while the normal blue thirst meter is still comfortable.
        if survival.0.thirst() < CRITICAL_THIRST {
            let severity = 1.0 - survival.0.thirst() / CRITICAL_THIRST;
            body.0
                .apply_external_drain(DEHYDRATION_BLOOD_DRAIN_PER_SEC * severity * dt);
        }
    }
}