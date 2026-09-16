//! Bevy wiring for hunger and thirst. Hunger primarily affects movement,
//! thirst primarily affects health/vision, and critically low values now
//! compound each other so neglecting both becomes substantially more dangerous.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::{Body, DamageCause, LastDamageCause};
use crate::physics::CharacterController;

use super::state::SurvivalState;

const NORMAL_SPEED_CAP: f32 = 180.0;
const STARVING_SPEED_CAP: f32 = 58.0;
const HUNGER_SLOW_START: f32 = 0.68;
const STARVATION_DAMAGE_START: f32 = 0.14;
const CRITICAL_THIRST: f32 = 0.18;
const COMBINED_HARDSHIP_START: f32 = 0.35;

/// At zero hunger this costs roughly 1.8% blood volume per second.
const STARVATION_BLOOD_DRAIN_PER_SEC: f32 = 0.018;
/// At zero thirst this costs roughly 4.5% blood volume per second.
const DEHYDRATION_BLOOD_DRAIN_PER_SEC: f32 = 0.045;
/// Additional penalty when both needs are simultaneously low.
const COMBINED_HARDSHIP_DRAIN_PER_SEC: f32 = 0.015;

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
    mut cause: ResMut<LastDamageCause>,
) {
    let dt = time.delta_secs();
    for (survival, mut body, mut velocity) in &mut query {
        let hunger = survival.0.hunger();
        let thirst = survival.0.thirst();

        // Hunger progressively removes mobility rather than waiting for 0%.
        if hunger < HUNGER_SLOW_START {
            let severity = 1.0 - hunger / HUNGER_SLOW_START;
            let cap = NORMAL_SPEED_CAP
                - (NORMAL_SPEED_CAP - STARVING_SPEED_CAP) * severity.clamp(0.0, 1.0);
            velocity.x = velocity.x.clamp(-cap, cap);
        }

        // Starvation is slower than dehydration, but it can now kill if ignored.
        if hunger < STARVATION_DAMAGE_START {
            let severity = 1.0 - hunger / STARVATION_DAMAGE_START;
            body.0.apply_external_drain(
                STARVATION_BLOOD_DRAIN_PER_SEC * severity.clamp(0.0, 1.0) * dt,
            );
        }

        // Dehydration becomes dangerous sooner than before and remains the faster
        // survival-related health threat.
        if thirst < CRITICAL_THIRST {
            let severity = 1.0 - thirst / CRITICAL_THIRST;
            body.0.apply_external_drain(
                DEHYDRATION_BLOOD_DRAIN_PER_SEC * severity.clamp(0.0, 1.0) * dt,
            );
            cause.0 = DamageCause::Dehydration;
        }

        // The important correlation: being low on both is worse than simply adding
        // two unrelated bars. This encourages players to manage food and water as
        // one survival problem instead of waiting for either meter to hit zero.
        if hunger < COMBINED_HARDSHIP_START && thirst < COMBINED_HARDSHIP_START {
            let hunger_severity = 1.0 - hunger / COMBINED_HARDSHIP_START;
            let thirst_severity = 1.0 - thirst / COMBINED_HARDSHIP_START;
            let combined = hunger_severity.min(thirst_severity).clamp(0.0, 1.0);
            body.0
                .apply_external_drain(COMBINED_HARDSHIP_DRAIN_PER_SEC * combined * dt);
        }
    }
}
