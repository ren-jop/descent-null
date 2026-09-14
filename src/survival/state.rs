//! Hunger and thirst: two meters that drain over real time regardless of
//! what else is happening. Engine-agnostic — wrapped as a `Survival`
//! component in `integration.rs`.
//!
//! Numbers are placeholder-tuned for a short playtest session, not a real
//! survival-game pace: full-to-empty takes minutes, not in-game days,
//! specifically so the consequence is visible without waiting around.
//! Revisit once there's a day/night cycle to peg these against instead.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurvivalState {
    /// 1.0 = fully fed, 0.0 = starving.
    hunger: f32,
    /// 1.0 = fully hydrated, 0.0 = dehydrated.
    thirst: f32,
}

/// Hunger empties over 5 minutes of continuous play.
const HUNGER_DRAIN_PER_SEC: f32 = 1.0 / 300.0;
/// Thirst empties faster than hunger — 3 minutes.
const THIRST_DRAIN_PER_SEC: f32 = 1.0 / 180.0;

impl Default for SurvivalState {
    fn default() -> Self {
        Self {
            hunger: 1.0,
            thirst: 1.0,
        }
    }
}

impl SurvivalState {
    pub fn hunger(&self) -> f32 {
        self.hunger
    }

    pub fn thirst(&self) -> f32 {
        self.thirst
    }

    pub fn is_starving(&self) -> bool {
        self.hunger <= 0.0
    }

    pub fn is_dehydrated(&self) -> bool {
        self.thirst <= 0.0
    }

    /// Drains both meters by `dt` seconds' worth. Call once per frame.
    pub fn tick(&mut self, dt: f32) {
        self.hunger = (self.hunger - HUNGER_DRAIN_PER_SEC * dt).max(0.0);
        self.thirst = (self.thirst - THIRST_DRAIN_PER_SEC * dt).max(0.0);
    }

    /// Restores both meters to full — used on player reset. A real
    /// food/water item system (milestone 3) will restore these partially
    /// instead; there's no eating or drinking yet, just the drain.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_full() {
        let survival = SurvivalState::default();
        assert_eq!(survival.hunger(), 1.0);
        assert_eq!(survival.thirst(), 1.0);
        assert!(!survival.is_starving());
        assert!(!survival.is_dehydrated());
    }

    #[test]
    fn ticking_drains_both_meters() {
        let mut survival = SurvivalState::default();
        survival.tick(30.0);
        assert!(survival.hunger() < 1.0);
        assert!(survival.thirst() < 1.0);
    }

    #[test]
    fn thirst_drains_faster_than_hunger() {
        let mut survival = SurvivalState::default();
        survival.tick(60.0);
        assert!(survival.thirst() < survival.hunger());
    }

    #[test]
    fn meters_do_not_go_negative() {
        let mut survival = SurvivalState::default();
        survival.tick(10_000.0);
        assert_eq!(survival.hunger(), 0.0);
        assert_eq!(survival.thirst(), 0.0);
        assert!(survival.is_starving());
        assert!(survival.is_dehydrated());
    }

    #[test]
    fn reset_restores_full() {
        let mut survival = SurvivalState::default();
        survival.tick(10_000.0);
        survival.reset();
        assert_eq!(survival.hunger(), 1.0);
        assert_eq!(survival.thirst(), 1.0);
    }
}
