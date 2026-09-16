//! Simple survival needs: hunger and thirst drain over real time.
//! They are intentionally direct and readable: no separate stamina meter.
//! Low needs impair movement; critical dehydration can reduce blood volume
//! through the Bevy integration layer.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurvivalState {
    /// 1.0 = fully fed, 0.0 = starving.
    hunger: f32,
    /// 1.0 = fully hydrated, 0.0 = critically dehydrated.
    thirst: f32,
}

/// Placeholder tuning for a short playtest session.
const HUNGER_DRAIN_PER_SEC: f32 = 1.0 / 180.0;
const THIRST_DRAIN_PER_SEC: f32 = 1.0 / 150.0;
/// Below this point a need starts impairing movement.
const HARDSHIP_THRESHOLD: f32 = 0.40;

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

    pub fn hunger_hardship(&self) -> f32 {
        if self.hunger >= HARDSHIP_THRESHOLD {
            0.0
        } else {
            1.0 - self.hunger / HARDSHIP_THRESHOLD
        }
    }

    pub fn thirst_hardship(&self) -> f32 {
        if self.thirst >= HARDSHIP_THRESHOLD {
            0.0
        } else {
            1.0 - self.thirst / HARDSHIP_THRESHOLD
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.hunger = (self.hunger - HUNGER_DRAIN_PER_SEC * dt).max(0.0);
        self.thirst = (self.thirst - THIRST_DRAIN_PER_SEC * dt).max(0.0);
    }

    pub fn eat(&mut self, amount: f32) {
        self.hunger = (self.hunger + amount).min(1.0);
    }

    pub fn drink(&mut self, amount: f32) {
        self.thirst = (self.thirst + amount).min(1.0);
    }

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

    #[test]
    fn eating_restores_hunger_only() {
        let mut survival = SurvivalState::default();
        survival.tick(200.0);
        let thirst_before = survival.thirst();
        survival.eat(0.3);
        assert!(survival.hunger() > 0.0);
        assert_eq!(survival.thirst(), thirst_before);
    }

    #[test]
    fn eating_and_drinking_cannot_exceed_full() {
        let mut survival = SurvivalState::default();
        survival.eat(0.5);
        survival.drink(0.5);
        assert_eq!(survival.hunger(), 1.0);
        assert_eq!(survival.thirst(), 1.0);
    }

    #[test]
    fn hardship_ramps_up_below_threshold() {
        let mut survival = SurvivalState::default();
        survival.tick(150.0);
        assert!(survival.hunger_hardship() > 0.0);
        assert!(survival.thirst_hardship() > 0.0);
    }
}