//! Bevy wiring for hunger/thirst/stamina. Survival needs change what the
//! player can do before they ever masquerade as unexplained health damage.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::CharacterController;

use super::state::SurvivalState;

const EXHAUSTED_SPEED_CAP: f32 = 80.0;
const NORMAL_SPEED_CAP: f32 = 180.0;
const LOW_NEEDS_MIN_SPEED_CAP: f32 = 115.0;

#[derive(Component, Default)]
pub struct Survival(pub SurvivalState);

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tick_survival, tick_stamina, enforce_survival_penalties).chain());
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

fn enforce_survival_penalties(
    mut query: Query<(&Survival, &mut LinearVelocity), With<CharacterController>>,
) {
    for (survival, mut velocity) in &mut query {
        if survival.0.is_exhausted() {
            velocity.x = velocity.x.clamp(-EXHAUSTED_SPEED_CAP, EXHAUSTED_SPEED_CAP);
            continue;
        }

        // Hunger/thirst already slow stamina recovery. Below the hardship
        // threshold they now also gradually reduce horizontal speed, making
        // the meters matter before they hit zero without touching health.
        let hardship = survival
            .0
            .hunger_hardship()
            .max(survival.0.thirst_hardship());
        if hardship > 0.0 {
            let cap = NORMAL_SPEED_CAP
                - (NORMAL_SPEED_CAP - LOW_NEEDS_MIN_SPEED_CAP) * hardship;
            velocity.x = velocity.x.clamp(-cap, cap);
        }
    }
}
