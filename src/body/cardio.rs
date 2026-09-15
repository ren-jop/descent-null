//! Cardiovascular state: the sink that wounds (and, later, starvation/
//! dehydration) drain into. Deliberately just one number for now — real
//! heart rate / shock / infection are future work — but it's a real
//! number with real consequences: run it to zero and the player dies.

/// Blood volume is 0..=1 (fraction of a full, healthy supply).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cardio {
    blood_volume: f32,
}

/// Below this fraction, the player is unconscious (movement disabled).
pub const KO_THRESHOLD: f32 = 0.40;
/// Below this fraction, the player is dead.
pub const DEATH_THRESHOLD: f32 = 0.15;
/// Passive recovery — stands in for clotting until a real medical/treatment
/// system exists (docs/REBUILD_PLAN.md milestone 6). Small on purpose: it
/// should not out-heal an active wound, only slowly undo old damage once
/// nothing is actively bleeding.
const PASSIVE_REGEN_PER_SEC: f32 = 0.01;

impl Default for Cardio {
    fn default() -> Self {
        Self { blood_volume: 1.0 }
    }
}

impl Cardio {
    pub fn blood_volume(&self) -> f32 {
        self.blood_volume
    }

    /// Removes blood volume (from bleeding, starvation, dehydration —
    /// anything). Never goes below zero.
    pub fn apply_drain(&mut self, amount: f32) {
        self.blood_volume = (self.blood_volume - amount).max(0.0);
    }

    /// Slow passive recovery, clamped at full. Called once per tick
    /// alongside whatever drains applied that same tick, so an actively
    /// bleeding wound still nets a loss.
    pub fn regen(&mut self, dt: f32) {
        self.blood_volume = (self.blood_volume + PASSIVE_REGEN_PER_SEC * dt).min(1.0);
    }

    /// Direct restoration — a medkit's effect, not passive regen.
    pub fn heal(&mut self, amount: f32) {
        self.blood_volume = (self.blood_volume + amount).min(1.0);
    }

    pub fn is_ko(&self) -> bool {
        self.blood_volume < KO_THRESHOLD
    }

    pub fn is_dead(&self) -> bool {
        self.blood_volume < DEATH_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_full_and_conscious() {
        let cardio = Cardio::default();
        assert_eq!(cardio.blood_volume(), 1.0);
        assert!(!cardio.is_ko());
        assert!(!cardio.is_dead());
    }

    #[test]
    fn draining_past_ko_threshold_knocks_out() {
        let mut cardio = Cardio::default();
        cardio.apply_drain(0.65);
        assert!(cardio.is_ko());
        assert!(!cardio.is_dead(), "KO'd is not the same as dead");
    }

    #[test]
    fn draining_past_death_threshold_kills() {
        let mut cardio = Cardio::default();
        cardio.apply_drain(0.9);
        assert!(cardio.is_dead());
    }

    #[test]
    fn drain_never_goes_negative() {
        let mut cardio = Cardio::default();
        cardio.apply_drain(5.0);
        assert_eq!(cardio.blood_volume(), 0.0);
    }

    #[test]
    fn passive_regen_cannot_exceed_full() {
        let mut cardio = Cardio::default();
        cardio.regen(1000.0);
        assert_eq!(cardio.blood_volume(), 1.0);
    }

    #[test]
    fn regen_recovers_slowly_after_a_drain() {
        let mut cardio = Cardio::default();
        cardio.apply_drain(0.2);
        cardio.regen(1.0);
        assert!(cardio.blood_volume() > 0.8 && cardio.blood_volume() < 1.0);
    }
}
