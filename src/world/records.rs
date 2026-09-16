//! Expo-friendly session leaderboard. Records live only for the current app
//! launch, so a shared computer can rotate through many players without an
//! account or stale personal data carrying over from a previous day.

use bevy::prelude::*;

const MAX_RECORDS: usize = 5;

#[derive(Clone, Debug, PartialEq)]
pub struct RunRecord {
    pub name: String,
    pub seconds: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SessionTimes {
    pub records: Vec<RunRecord>,
}

impl SessionTimes {
    fn normalise(&mut self) {
        self.records.retain(|record| record.seconds.is_finite() && record.seconds > 0.0);
        self.records.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
        self.records.truncate(MAX_RECORDS);
    }

    pub fn qualifies(&self, seconds: f32) -> bool {
        if !seconds.is_finite() || seconds <= 0.0 {
            return false;
        }
        self.records.len() < MAX_RECORDS
            || self.records.last().map(|record| seconds < record.seconds).unwrap_or(true)
    }

    pub fn record(&mut self, name: impl Into<String>, seconds: f32) {
        let mut name = name.into().trim().to_string();
        if name.is_empty() {
            name = "PLAYER".to_string();
        }
        name.truncate(12);
        self.records.push(RunRecord { name, seconds });
        self.normalise();
    }

    pub fn formatted(&self) -> String {
        if self.records.is_empty() {
            return "No completed runs yet".to_string();
        }
        self.records
            .iter()
            .enumerate()
            .map(|(index, record)| format!("{}. {:<12}  {:.1}s", index + 1, record.name, record.seconds))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn best(&self) -> Option<&RunRecord> {
        self.records.first()
    }
}

/// Temporary compatibility resource for the older mission-complete overlay.
/// The actual leaderboard is session-only and lives in `SessionTimes`.
#[derive(Resource, Default)]
pub struct BestTimes;

impl BestTimes {
    pub fn formatted(&self) -> String {
        "Session leaderboard: press L".to_string()
    }
}

pub struct RecordsPlugin;

impl Plugin for RecordsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SessionTimes>()
            .init_resource::<BestTimes>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_five_times_qualify() {
        let mut session = SessionTimes::default();
        for index in 0..5 {
            assert!(session.qualifies(100.0 + index as f32));
            session.record(format!("P{index}"), 100.0 + index as f32);
        }
        assert_eq!(session.records.len(), 5);
    }

    #[test]
    fn slower_than_fifth_does_not_qualify() {
        let mut session = SessionTimes::default();
        for (name, time) in [("A", 10.0), ("B", 20.0), ("C", 30.0), ("D", 40.0), ("E", 50.0)] {
            session.record(name, time);
        }
        assert!(!session.qualifies(60.0));
        assert!(session.qualifies(25.0));
    }

    #[test]
    fn record_keeps_fastest_five_and_names() {
        let mut session = SessionTimes::default();
        session.record("SLOW", 60.0);
        session.record("FAST", 10.0);
        session.record("MID", 30.0);
        assert_eq!(session.best().unwrap().name, "FAST");
        assert!(session.formatted().contains("FAST"));
    }
}
