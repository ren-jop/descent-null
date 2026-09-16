mod hud;
mod leaderboard;

use bevy::prelude::*;

pub use leaderboard::LeaderboardState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((hud::HudPlugin, leaderboard::LeaderboardPlugin));
    }
}
