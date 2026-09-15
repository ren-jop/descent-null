//! Bevy wiring for hunger/thirst/stamina. Depends on `body` and `physics`
//! (starvation/dehydration drain body's blood-volume pipeline; exhaustion
//! caps movement speed) — never the other way around, matching the
//! dependency graph in docs/REBUILD_PLAN.md.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::physics::CharacterController;

use super::state::SurvivalState;

/// Blood-volume-equivalent drained per second while starving.
const STARVATION_DRAIN_PER_SEC: f32 = 0.015;
/// Dehydration is worse than starvation over the same timescale.
const DEHYDRATION_DRAIN_PER_SEC: f32 = 0.02;
/// horizontal speed cap while exhausted. placeholder — tune against feel.
const EXHAUSTED_SPEED_CAP: f32 = 80.0;

#[derive(Component, Default)]
pub struct Survival(pub SurvivalState);

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (tick_survival, tick_stamina, apply_starvation_to_body, enforce_exhaustion).chain(),
        );
    }
}

fn tick_survival(time: Res<Time>, mut query: Query<&mut Survival>) {
    let dt = time.delta_secs();
    for mut survival in &mut query {
        survival.0.tick(dt);
    }
}

fn tick_stamina(time: Res<Time>, keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Survival>) {
    let dt = time.delta_secs();
    let exerting = keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD)
        || keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::ArrowRight);
    for mut survival in &mut query {
        survival.0.tick_stamina(dt, exerting);
    }
}

fn apply_starvation_to_body(time: Res<Time>, mut query: Query<(&Survival, &mut Body)>) {
    let dt = time.delta_secs();
    for (survival, mut body) in &mut query {
        if survival.0.is_starving() {
            body.0.apply_external_drain(STARVATION_DRAIN_PER_SEC * dt);
        }
        if survival.0.is_dehydrated() {
            body.0.apply_external_drain(DEHYDRATION_DRAIN_PER_SEC * dt);
        }
    }
}

fn enforce_exhaustion(
    mut query: Query<(&Survival, &mut LinearVelocity), With<CharacterController>>,
) {
    for (survival, mut velocity) in &mut query {
        if survival.0.is_exhausted() {
            velocity.x = velocity.x.clamp(-EXHAUSTED_SPEED_CAP, EXHAUSTED_SPEED_CAP);
        }
    }
}
