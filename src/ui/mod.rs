mod hud;
mod leaderboard;
mod survival_feedback;

use bevy::prelude::*;

pub use leaderboard::LeaderboardState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            hud::HudPlugin,
            leaderboard::LeaderboardPlugin,
            survival_feedback::SurvivalFeedbackPlugin,
        ));
    }
}
