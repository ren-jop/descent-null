//! Body simulation: per-region injuries plus a cardiovascular abstraction,
//! instead of a single HP number. This is pure simulation state -- no
//! rendering, no input.

use serde_like::Clamp;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Region {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}
pub const ALL_REGIONS: [Region; 6] = [
    Region::Head,
    Region::Torso,
    Region::LeftArm,
    Region::RightArm,
    Region::LeftLeg,
    Region::RightLeg,
];

#[derive(Clone, Copy, Debug)]
pub struct RegionState {
    /// 0 = destroyed, 100 = pristine
    pub condition: f32,
    /// active external bleeding rate (blood volume lost per tick)
    pub bleeding: f32,
    pub fracture: bool,
    /// 0-100 infection progress; treated with antiseptic/antibiotics
    pub infection: f32,
    pub pain: f32,
}

impl RegionState {
    fn healthy() -> Self {
        RegionState {
            condition: 100.0,
            bleeding: 0.0,
            fracture: false,
            infection: 0.0,
            pain: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cardiovascular {
    /// 0-100, 0 is fatal
    pub blood_volume: f32,
    pub heart_rate: f32,
    /// 0-100, 0 is unconscious
    pub consciousness: f32,
    pub shock: f32,
}

impl Cardiovascular {
    fn healthy() -> Self {
        Cardiovascular {
            blood_volume: 100.0,
            heart_rate: 70.0,
            consciousness: 100.0,
            shock: 0.0,
        }
    }
}

pub struct InjuryReport {
    pub region: Region,
    pub condition_lost: f32,
    pub caused_fracture: bool,
    pub caused_bleeding: f32,
}

pub struct BodyState {
    pub regions: [RegionState; 6],
    pub cardio: Cardiovascular,
    pub dead: bool,
    pub cause_of_death: Option<String>,
}

impl BodyState {
    pub fn new() -> Self {
        BodyState {
            regions: [RegionState::healthy(); 6],
            cardio: Cardiovascular::healthy(),
            dead: false,
            cause_of_death: None,
        }
    }

    pub fn region(&self, r: Region) -> &RegionState {
        &self.regions[r as usize]
    }

    /// Apply blunt/impact damage to a region (falls, hits, hazards).
    /// severity is roughly "how hard", unitless, typically 0-100.
    pub fn apply_injury(&mut self, r: Region, severity: f32) -> InjuryReport {
        let idx = r as usize;
        let region = &mut self.regions[idx];
        let condition_lost = severity.clamp(0.0, 100.0);
        region.condition = (region.condition - condition_lost).clamp01();
        region.pain = (region.pain + severity * 0.8).clamp(0.0, 100.0);

        let mut caused_fracture = false;
        // harder hits on a limb/head have a chance-like (deterministic on
        // severity, not RNG, so behaviour stays testable) chance of a
        // fracture once condition drops low enough
        if severity > 35.0 && region.condition < 60.0 && !region.fracture {
            region.fracture = true;
            caused_fracture = true;
        }

        let bleeding_added = (severity * 0.06).max(0.0);
        region.bleeding += bleeding_added;

        InjuryReport {
            region: r,
            condition_lost,
            caused_fracture,
            caused_bleeding: bleeding_added,
        }
    }

    /// Advance the simulation by one tick. Bleeding drains blood volume;
    /// blood volume and shock drive consciousness; consciousness hitting
    /// zero (or blood volume hitting zero directly) is fatal.
    pub fn tick(&mut self) {
        if self.dead {
            return;
        }

        let mut total_bleeding = 0.0;
        for region in &mut self.regions {
            total_bleeding += region.bleeding;
            // infection creeps up slowly on any region with broken skin
            // (bleeding) or a fracture that hasn't been treated
            if region.bleeding > 0.0 || region.fracture {
                region.infection = (region.infection + 0.15).clamp(0.0, 100.0);
            }
            // pain fades slowly on its own even without treatment
            region.pain = (region.pain - 0.3).clamp(0.0, 100.0);
        }

        self.cardio.blood_volume = (self.cardio.blood_volume - total_bleeding).clamp01();

        // heart rate compensates for blood loss (rises), then that
        // compensation itself becomes a cost (shock) once blood loss is severe
        let deficit = 100.0 - self.cardio.blood_volume;
        self.cardio.heart_rate = 70.0 + deficit * 0.6;
        if deficit > 40.0 {
            self.cardio.shock = (self.cardio.shock + (deficit - 40.0) * 0.05).clamp(0.0, 100.0);
        } else {
            self.cardio.shock = (self.cardio.shock - 0.5).clamp(0.0, 100.0);
        }

        // consciousness driven by blood volume and shock together
        let target_consciousness = (self.cardio.blood_volume - self.cardio.shock * 0.5).clamp01();
        self.cardio.consciousness += (target_consciousness - self.cardio.consciousness) * 0.2;
        self.cardio.consciousness = self.cardio.consciousness.clamp01();

        if self.cardio.blood_volume <= 0.0 {
            self.dead = true;
            self.cause_of_death = Some("catastrophic blood loss".to_string());
        } else if self.cardio.consciousness <= 0.0 {
            self.dead = true;
            self.cause_of_death = Some("loss of consciousness with no one to help".to_string());
        } else {
            let total_infection: f32 = self.regions.iter().map(|r| r.infection).sum();
            if total_infection > 500.0 {
                self.dead = true;
                self.cause_of_death = Some("untreated systemic infection".to_string());
            }
        }
    }

    /// Overall movement penalty (0.0 = full speed, 1.0 = can't move) driven
    /// by leg condition/fracture and general consciousness.
    pub fn movement_penalty(&self) -> f32 {
        let leg_penalty = [Region::LeftLeg, Region::RightLeg]
            .iter()
            .map(|r| {
                let region = self.region(*r);
                let mut p = 1.0 - region.condition / 100.0;
                if region.fracture {
                    p = (p + 0.5).min(1.0);
                }
                p
            })
            .sum::<f32>()
            / 2.0;
        let consciousness_penalty = 1.0 - self.cardio.consciousness / 100.0;
        (leg_penalty * 0.7 + consciousness_penalty * 0.6).clamp(0.0, 1.0)
    }

    // --- medical treatment ---

    pub fn apply_bandage(&mut self, r: Region) {
        self.regions[r as usize].bleeding = (self.regions[r as usize].bleeding - 0.8).max(0.0);
    }

    pub fn apply_splint(&mut self, r: Region) {
        self.regions[r as usize].fracture = false;
    }

    pub fn apply_antiseptic(&mut self, r: Region) {
        self.regions[r as usize].infection = (self.regions[r as usize].infection - 30.0).max(0.0);
    }

    pub fn apply_painkiller(&mut self, r: Region) {
        self.regions[r as usize].pain = (self.regions[r as usize].pain - 40.0).max(0.0);
    }

    pub fn apply_clotting_agent(&mut self, r: Region) {
        self.regions[r as usize].bleeding = 0.0;
    }
}

/// Tiny local helper trait so we don't need an external crate just for a
/// 0..=100 clamp shorthand.
mod serde_like {
    pub trait Clamp {
        fn clamp01(self) -> f32;
    }
    impl Clamp for f32 {
        fn clamp01(self) -> f32 {
            self.clamp(0.0, 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_body_starts_stable() {
        let mut b = BodyState::new();
        for _ in 0..50 {
            b.tick();
        }
        assert!(!b.dead);
        assert!(b.cardio.blood_volume > 95.0);
    }

    #[test]
    fn severe_untreated_bleeding_is_eventually_fatal() {
        let mut b = BodyState::new();
        b.apply_injury(Region::Torso, 90.0);
        b.apply_injury(Region::LeftLeg, 90.0);
        let mut ticks = 0;
        while !b.dead && ticks < 1000 {
            b.tick();
            ticks += 1;
        }
        assert!(b.dead, "expected fatal outcome from severe untreated bleeding");
        assert!(b.cause_of_death.is_some());
    }

    #[test]
    fn bandaging_a_wound_slows_or_stops_its_bleeding() {
        let mut b = BodyState::new();
        b.apply_injury(Region::Torso, 60.0);
        let bleeding_before = b.region(Region::Torso).bleeding;
        b.apply_bandage(Region::Torso);
        assert!(b.region(Region::Torso).bleeding < bleeding_before);
    }

    #[test]
    fn treated_injury_is_survivable() {
        let mut b = BodyState::new();
        b.apply_injury(Region::Torso, 60.0);
        b.apply_bandage(Region::Torso);
        b.apply_bandage(Region::Torso); // stop it fully
        b.apply_clotting_agent(Region::Torso);
        for _ in 0..300 {
            b.tick();
        }
        assert!(!b.dead, "a treated, non-bleeding injury should not be fatal");
    }

    #[test]
    fn fracture_and_leg_damage_impair_movement() {
        let mut b = BodyState::new();
        let before = b.movement_penalty();
        b.apply_injury(Region::LeftLeg, 80.0);
        let after = b.movement_penalty();
        assert!(after > before);
    }

    #[test]
    fn splinting_clears_a_fracture() {
        let mut b = BodyState::new();
        b.apply_injury(Region::RightArm, 80.0);
        assert!(b.region(Region::RightArm).fracture);
        b.apply_splint(Region::RightArm);
        assert!(!b.region(Region::RightArm).fracture);
    }
}
