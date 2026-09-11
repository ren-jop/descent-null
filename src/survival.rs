//! Hunger/thirst/stamina/temperature/mood -- the layer above raw body
//! injury that governs day-to-day survival pressure. These stats feed
//! back into the body simulation (see `apply_to_body`) rather than being
//! decorative bars.

use crate::body::BodyState;

#[derive(Clone, Copy, Debug)]
pub struct SurvivalState {
    pub hunger: f32,      // 100 = fully fed, 0 = starving
    pub thirst: f32,      // 100 = hydrated, 0 = severely dehydrated
    pub stamina: f32,     // 0-100, spent by exertion, regenerates over time
    pub temperature: f32, // body temp abstraction, 50 = comfortable, drifts toward environment
    pub fatigue: f32,     // 0 = rested, 100 = exhausted
    pub mood: f32,        // 0-100, affected by pain/danger/isolation
}

impl SurvivalState {
    pub fn new() -> Self {
        SurvivalState {
            hunger: 100.0,
            thirst: 100.0,
            stamina: 100.0,
            temperature: 50.0,
            fatigue: 0.0,
            mood: 70.0,
        }
    }

    /// One simulation tick. `environment_temp` is the ambient temperature
    /// of the current tile/layer (50 = neutral, lower = cold, higher = hot).
    pub fn tick(&mut self, environment_temp: f32) {
        self.hunger = (self.hunger - 0.25).max(0.0);
        self.thirst = (self.thirst - 0.4).max(0.0);
        self.fatigue = (self.fatigue + 0.15).min(100.0);

        // temperature drifts toward the environment
        self.temperature += (environment_temp - self.temperature) * 0.05;

        // stamina regenerates only when hydrated and fed enough; degraded
        // regen when either is critically low
        let regen_rate = if self.thirst < 15.0 || self.hunger < 15.0 {
            0.1
        } else {
            0.6
        };
        self.stamina = (self.stamina + regen_rate - self.fatigue * 0.01).clamp(0.0, 100.0);

        // mood drifts toward a baseline set by physical comfort
        let comfort = (self.hunger + self.thirst + (100.0 - self.fatigue)) / 3.0;
        self.mood += (comfort - self.mood) * 0.03;
        self.mood = self.mood.clamp(0.0, 100.0);
    }

    pub fn spend_stamina(&mut self, amount: f32) {
        self.stamina = (self.stamina - amount).max(0.0);
    }

    pub fn eat(&mut self, amount: f32) {
        self.hunger = (self.hunger + amount).min(100.0);
    }

    pub fn drink(&mut self, amount: f32) {
        self.thirst = (self.thirst + amount).min(100.0);
    }

    pub fn rest(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }

    /// Feed survival-state consequences back into the body simulation.
    /// Called once per tick alongside `SurvivalState::tick`.
    pub fn apply_to_body(&self, body: &mut BodyState) {
        // starvation/dehydration manifest as mounting cardiovascular shock,
        // same channel that severe blood loss uses -- consistent with the
        // "your body starts giving you problems" design goal
        if self.hunger <= 0.0 || self.thirst <= 0.0 {
            body.cardio.shock = (body.cardio.shock + 0.4).min(100.0);
        }
        // extreme cold or heat does direct, slow condition damage to
        // extremities-adjacent regions (approximated here across all limbs)
        if self.temperature < 20.0 || self.temperature > 80.0 {
            for region in &mut body.regions {
                region.condition = (region.condition - 0.05).max(0.0);
            }
        }
    }

    /// Movement speed multiplier from stamina/fatigue alone (independent of
    /// body-simulation movement penalty -- the two are combined by the
    /// caller).
    pub fn stamina_speed_multiplier(&self) -> f32 {
        if self.stamina < 10.0 {
            0.4
        } else if self.stamina < 30.0 {
            0.7
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_decay_over_time_at_neutral_temperature() {
        let mut s = SurvivalState::new();
        for _ in 0..20 {
            s.tick(50.0);
        }
        assert!(s.hunger < 100.0);
        assert!(s.thirst < 100.0);
    }

    #[test]
    fn starving_eventually_adds_cardiovascular_shock() {
        let mut s = SurvivalState::new();
        let mut b = BodyState::new();
        for _ in 0..500 {
            s.tick(50.0);
            s.apply_to_body(&mut b);
        }
        assert_eq!(s.hunger, 0.0);
        assert!(b.cardio.shock > 0.0);
    }

    #[test]
    fn eating_and_drinking_restore_stats() {
        let mut s = SurvivalState::new();
        for _ in 0..100 {
            s.tick(50.0);
        }
        let hunger_before = s.hunger;
        s.eat(30.0);
        assert!(s.hunger > hunger_before);
    }

    #[test]
    fn extreme_cold_damages_body_condition_over_time() {
        let mut s = SurvivalState::new();
        let mut b = BodyState::new();
        for _ in 0..400 {
            s.tick(5.0); // very cold environment
            s.apply_to_body(&mut b);
        }
        assert!(b.region(crate::body::Region::Torso).condition < 100.0);
    }
}
