//! Local run records: persistent personal bests plus best times from the
//! current launch session. No account/network dependency.

use std::fs;

use bevy::prelude::*;

use super::RunStats;

const RECORD_PATH: &str = ".descent_null_best_times";
const MAX_RECORDS: usize = 5;

fn normalise(times: &mut Vec<f32>) {
    times.retain(|time| time.is_finite() && *time > 0.0);
    times.sort_by(|a, b| a.total_cmp(b));
    times.truncate(MAX_RECORDS);
}

fn formatted(times: &[f32]) -> String {
    if times.is_empty() {
        return "No completed runs yet".to_string();
    }
    times
        .iter()
        .enumerate()
        .map(|(index, seconds)| format!("{}. {:.1}s", index + 1, seconds))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Resource, Clone, Debug, Default)]
pub struct BestTimes {
    pub times: Vec<f32>,
}

impl BestTimes {
    fn load() -> Self {
        let mut times = fs::read_to_string(RECORD_PATH)
            .ok()
            .into_iter()
            .flat_map(|text| {
                text.lines()
                    .filter_map(|line| line.trim().parse::<f32>().ok())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        normalise(&mut times);
        Self { times }
    }

    fn record(&mut self, seconds: f32) {
        self.times.push(seconds);
        normalise(&mut self.times);
        let text = self
            .times
            .iter()
            .map(|time| format!("{time:.3}"))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write(RECORD_PATH, format!("{text}\n"));
    }

    pub fn formatted(&self) -> String {
        formatted(&self.times)
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SessionTimes {
    pub times: Vec<f32>,
}

impl SessionTimes {
    fn record(&mut self, seconds: f32) {
        self.times.push(seconds);
        normalise(&mut self.times);
    }

    pub fn formatted(&self) -> String {
        formatted(&self.times)
    }
}

pub struct RecordsPlugin;

impl Plugin for RecordsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BestTimes::load())
            .init_resource::<SessionTimes>()
            .add_systems(Update, record_completed_run);
    }
}

fn record_completed_run(
    stats: Res<RunStats>,
    mut best: ResMut<BestTimes>,
    mut session: ResMut<SessionTimes>,
    mut recorded_this_run: Local<bool>,
) {
    if !stats.extracted {
        *recorded_this_run = false;
        return;
    }
    if !*recorded_this_run {
        best.record(stats.elapsed_secs);
        session.record(stats.elapsed_secs);
        *recorded_this_run = true;
    }
}
