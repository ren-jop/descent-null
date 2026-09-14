//! Bevy wiring for hunger/thirst. Depends on `body` (starvation and
//! dehydration drain the same blood-volume/consciousness pipeline that
//! wounds do) — never the other way around, matching the dependency graph
//! in docs/REBUILD_PLAN.md.

use bevy::prelude::*;

use crate::body::Body;

use super::state::SurvivalState;

/// Blood-volume-equivalent drained per second while starving.
const STARVATION_DRAIN_PER_SEC: f32 = 0.015;
/// Dehydration is worse than starvation over the same timescale.
const DEHYDRATION_DRAIN_PER_SEC: f32 = 0.02;

#[derive(Component, Default)]
pub struct Survival(pub SurvivalState);

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tick_survival, apply_starvation_to_body).chain());
    }
}

fn tick_survival(time: Res<Time>, mut query: Query<&mut Survival>) {
    let dt = time.delta_secs();
    for mut survival in &mut query {
        survival.0.tick(dt);
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
