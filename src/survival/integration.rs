//! Bevy wiring for hunger and thirst. Hunger is the movement-pressure need;
//! thirst is the vision/health-pressure need. Critical dehydration drains
//! blood volume quickly enough that ignoring water becomes an urgent problem.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::{Body, DamageCause, LastDamageCause};
use crate::physics::CharacterController;

use super::state::SurvivalState;

const NORMAL_SPEED_CAP: f32 = 180.0;
const STARVING_SPEED_CAP: f32 = 65.0;
const HUNGER_SLOW_START: f32 = 0.65;
const CRITICAL_THIRST: f32 = 0.12;
/// At zero thirst this removes roughly 4% blood volume per second. The effect
/// ramps in below 12% so the field log/vision warning arrives before damage.
const DEHYDRATION_BLOOD_DRAIN_PER_SEC: f32 = 0.04;

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
        // Hunger has one simple, readable consequence: the lower it gets,
        // the slower horizontal movement becomes. Thirst does not also slow
        // movement; it owns the vision + dehydration-health consequences.
        let hunger = survival.0.hunger();
        if hunger < HUNGER_SLOW_START {
            let severity = 1.0 - hunger / HUNGER_SLOW_START;
            let cap = NORMAL_SPEED_CAP
                - (NORMAL_SPEED_CAP - STARVING_SPEED_CAP) * severity.clamp(0.0, 1.0);
            velocity.x = velocity.x.clamp(-cap, cap);
        }

        if survival.0.thirst() < CRITICAL_THIRST {
            let severity = 1.0 - survival.0.thirst() / CRITICAL_THIRST;
            body.0
                .apply_external_drain(DEHYDRATION_BLOOD_DRAIN_PER_SEC * severity * dt);
            cause.0 = DamageCause::Dehydration;
        }
    }
}
