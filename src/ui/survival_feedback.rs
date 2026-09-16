use bevy::prelude::*;

use crate::items::LastEvent;
use crate::player::Player;
use crate::survival::Survival;

#[derive(Resource, Default)]
struct SurvivalFeedbackState {
    hunger_stage: u8,
    thirst_stage: u8,
}

pub struct SurvivalFeedbackPlugin;

impl Plugin for SurvivalFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurvivalFeedbackState>()
            .add_systems(Update, update_survival_feedback);
    }
}

fn hunger_stage(value: f32) -> u8 {
    if value <= 0.15 {
        3
    } else if value <= 0.35 {
        2
    } else if value <= 0.65 {
        1
    } else {
        0
    }
}

fn thirst_stage(value: f32) -> u8 {
    if value <= 0.12 {
        3
    } else if value <= 0.25 {
        2
    } else if value <= 0.50 {
        1
    } else {
        0
    }
}

fn update_survival_feedback(
    player: Query<&Survival, With<Player>>,
    mut state: ResMut<SurvivalFeedbackState>,
    mut last: ResMut<LastEvent>,
) {
    let Ok(survival) = player.single() else {
        return;
    };

    let new_thirst = thirst_stage(survival.0.thirst());
    let new_hunger = hunger_stage(survival.0.hunger());

    if new_thirst > state.thirst_stage && last.remaining <= 0.05 {
        let message = match new_thirst {
            1 => "THIRST  VISION IS STARTING TO NARROW",
            2 => "VERY THIRSTY  VISION IS HEAVILY IMPAIRED",
            _ => "DEHYDRATED  VISION CRITICAL + HEALTH FALLING",
        };
        last.show(message);
    } else if new_hunger > state.hunger_stage && last.remaining <= 0.05 {
        let message = match new_hunger {
            1 => "HUNGER  MOVEMENT IS STARTING TO SLOW",
            2 => "VERY HUNGRY  MOVEMENT SPEED REDUCED",
            _ => "STARVING  MOVEMENT SEVERELY REDUCED",
        };
        last.show(message);
    }

    state.thirst_stage = new_thirst;
    state.hunger_stage = new_hunger;
}
