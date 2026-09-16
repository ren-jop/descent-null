//! Small local personal-best leaderboard. Times are stored beside the game
//! so a player's best successful runs survive restarts without needing an
//! account or network service.

use std::fs;

use bevy::prelude::*;

use super::RunStats;

const RECORD_PATH: &str = ".descent_null_best_times";
const MAX_RECORDS: usize = 5;

#[derive(Resource, Clone, Debug, Default)]
pub struct BestTimes {
    pub times: Vec<f32>,
}

impl BestTimes {
    fn load() -> Self {
        let times = fs::read_to_string(RECORD_PATH)
            .ok()
            .into_iter()
            .flat_map(|text| {
                text.lines()
                    .filter_map(|line| line.trim().parse::<f32>().ok())
                    .collect::<Vec<_>>()
            })
            .filter(|time| time.is_finite() && *time > 0.0)
            .collect::<Vec<_>>();
        let mut best = Self { times };
        best.normalise();
        best
    }

    fn normalise(&mut self) {
        self.times.sort_by(|a, b| a.total_cmp(b));
        self.times.truncate(MAX_RECORDS);
    }

    fn record(&mut self, seconds: f32) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        self.times.push(seconds);
        self.normalise();
        let text = self
            .times
            .iter()
            .map(|time| format!("{time:.3}"))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write(RECORD_PATH, format!("{text}\n"));
    }

    pub fn formatted(&self) -> String {
        if self.times.is_empty() {
            return "No completed runs yet".to_string();
        }
        self.times
            .iter()
            .enumerate()
            .map(|(index, seconds)| format!("{}. {:.1}s", index + 1, seconds))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub struct RecordsPlugin;

impl Plugin for RecordsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BestTimes::load())
            .add_systems(Update, record_completed_run);
    }
}

fn record_completed_run(
    stats: Res<RunStats>,
    mut best: ResMut<BestTimes>,
    mut recorded_this_run: Local<bool>,
) {
    if !stats.extracted {
        *recorded_this_run = false;
        return;
    }
    if !*recorded_this_run {
        best.record(stats.elapsed_secs);
        *recorded_this_run = true;
    }
}
