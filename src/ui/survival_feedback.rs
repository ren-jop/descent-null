use bevy::prelude::*;

use crate::items::LastEvent;
use crate::player::Player;
use crate::survival::Survival;

#[derive(Resource, Default)]
struct SurvivalFeedbackState {
    hunger_stage: u8,
    thirst_stage: u8,
    combined_warning: bool,
}

pub struct SurvivalFeedbackPlugin;

impl Plugin for SurvivalFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurvivalFeedbackState>()
            .add_systems(Update, update_survival_feedback);
    }
}

fn hunger_stage(value: f32) -> u8 {
    if value <= 0.14 {
        3
    } else if value <= 0.35 {
        2
    } else if value <= 0.68 {
        1
    } else {
        0
    }
}

fn thirst_stage(value: f32) -> u8 {
    if value <= 0.18 {
        3
    } else if value <= 0.35 {
        2
    } else if value <= 0.55 {
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

    let hunger = survival.0.hunger();
    let thirst = survival.0.thirst();
    let new_thirst = thirst_stage(thirst);
    let new_hunger = hunger_stage(hunger);
    let combined = hunger <= 0.35 && thirst <= 0.35;

    if combined && !state.combined_warning && last.remaining <= 0.05 {
        last.show("HUNGER + THIRST LOW  NEEDS DRAIN EACH OTHER + HEALTH FALLS FASTER");
    } else if new_thirst > state.thirst_stage && last.remaining <= 0.05 {
        let message = match new_thirst {
            1 => "THIRST  VISION IS STARTING TO NARROW",
            2 => "VERY THIRSTY  HUNGER NOW DRAINS FASTER",
            _ => "DEHYDRATED  HEALTH IS FALLING FAST",
        };
        last.show(message);
    } else if new_hunger > state.hunger_stage && last.remaining <= 0.05 {
        let message = match new_hunger {
            1 => "HUNGER  MOVEMENT IS STARTING TO SLOW",
            2 => "VERY HUNGRY  THIRST NOW DRAINS FASTER",
            _ => "STARVING  MOVEMENT SEVERE + HEALTH FALLING",
        };
        last.show(message);
    }

    state.thirst_stage = new_thirst;
    state.hunger_stage = new_hunger;
    state.combined_warning = combined;
}
