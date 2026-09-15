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
    /// 1.0 = fully rested, 0.0 = exhausted.
    stamina: f32,
}

/// Hunger empties over 5 minutes of continuous play.
const HUNGER_DRAIN_PER_SEC: f32 = 1.0 / 300.0;
/// Thirst empties faster than hunger — 4 minutes.
const THIRST_DRAIN_PER_SEC: f32 = 1.0 / 240.0;
/// walking barely taxes this — 20s of continuous movement to empty it.
/// it's meant to matter after a lot of sustained action, not on every step.
const STAMINA_DRAIN_PER_SEC: f32 = 0.05;
const STAMINA_REGEN_PER_SEC: f32 = 0.25;
/// below this, movement gets a real penalty (see survival::integration).
pub const EXHAUSTED_THRESHOLD: f32 = 0.15;

impl Default for SurvivalState {
    fn default() -> Self {
        Self {
            hunger: 1.0,
            thirst: 1.0,
            stamina: 1.0,
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

    pub fn stamina(&self) -> f32 {
        self.stamina
    }

    pub fn is_starving(&self) -> bool {
        self.hunger <= 0.0
    }

    pub fn is_dehydrated(&self) -> bool {
        self.thirst <= 0.0
    }

    pub fn is_exhausted(&self) -> bool {
        self.stamina < EXHAUSTED_THRESHOLD
    }

    /// Drains both meters by `dt` seconds' worth. Call once per frame.
    pub fn tick(&mut self, dt: f32) {
        self.hunger = (self.hunger - HUNGER_DRAIN_PER_SEC * dt).max(0.0);
        self.thirst = (self.thirst - THIRST_DRAIN_PER_SEC * dt).max(0.0);
    }

    /// stamina drains while `exerting` (moving), regenerates otherwise.
    /// a starving or dehydrated body also regenerates stamina slower.
    pub fn tick_stamina(&mut self, dt: f32, exerting: bool) {
        if exerting {
            self.stamina = (self.stamina - STAMINA_DRAIN_PER_SEC * dt).max(0.0);
        } else {
            let regen_penalty = if self.is_starving() || self.is_dehydrated() { 0.6 } else { 1.0 };
            self.stamina = (self.stamina + STAMINA_REGEN_PER_SEC * regen_penalty * dt).min(1.0);
        }
    }

    /// Restores hunger — a food item's effect. Clamped at full.
    pub fn eat(&mut self, amount: f32) {
        self.hunger = (self.hunger + amount).min(1.0);
    }

    /// Restores thirst — a water item's effect. Clamped at full.
    pub fn drink(&mut self, amount: f32) {
        self.thirst = (self.thirst + amount).min(1.0);
    }

    /// Restores every meter to full — used on player reset (a fresh run),
    /// as opposed to `eat`/`drink` which restore hunger/thirst partially.
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
    fn exertion_drains_stamina() {
        let mut survival = SurvivalState::default();
        survival.tick_stamina(1.0, true);
        assert!(survival.stamina() < 1.0);
    }

    #[test]
    fn resting_regenerates_stamina() {
        let mut survival = SurvivalState::default();
        survival.tick_stamina(2.0, true);
        let drained = survival.stamina();
        survival.tick_stamina(2.0, false);
        assert!(survival.stamina() > drained);
    }

    #[test]
    fn low_stamina_is_exhausted() {
        let mut survival = SurvivalState::default();
        survival.tick_stamina(10.0, true);
        assert!(survival.is_exhausted());
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
}
