//! Bevy wiring for hunger/thirst/stamina. Survival needs should change
//! what the player can do before they ever masquerade as unexplained
//! health damage. Hunger/thirst therefore affect stamina recovery and
//! exhaustion; wounds remain the source of blood loss.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::CharacterController;

use super::state::SurvivalState;

/// horizontal speed cap while exhausted. placeholder — tune against feel.
const EXHAUSTED_SPEED_CAP: f32 = 80.0;

#[derive(Component, Default)]
pub struct Survival(pub SurvivalState);

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tick_survival, tick_stamina, enforce_exhaustion).chain());
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

fn enforce_exhaustion(
    mut query: Query<(&Survival, &mut LinearVelocity), With<CharacterController>>,
) {
    for (survival, mut velocity) in &mut query {
        if survival.0.is_exhausted() {
            velocity.x = velocity.x.clamp(-EXHAUSTED_SPEED_CAP, EXHAUSTED_SPEED_CAP);
        }
    }
}
