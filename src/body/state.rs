//! `BodyState`: the running list of wounds for one body. Engine-agnostic —
//! wrapped in a Bevy `Component` in `integration.rs` so this stays testable
//! without spinning up an `App`.

use super::cardio::Cardio;
use super::region::BodyRegion;
use super::wound::Wound;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BodyState {
    wounds: Vec<Wound>,
    cardio: Cardio,
}

impl BodyState {
    pub fn apply_wound(&mut self, wound: Wound) {
        self.wounds.push(wound);
    }

    pub fn wounds(&self) -> &[Wound] {
        &self.wounds
    }

    pub fn wound_count(&self) -> usize {
        self.wounds.len()
    }

    pub fn wound_count_in(&self, region: BodyRegion) -> usize {
        self.wounds.iter().filter(|w| w.region == region).count()
    }

    /// Sum of every untreated wound's ongoing pain contribution.
    pub fn total_pain(&self) -> f32 {
        self.wounds.iter().map(|w| w.pain).sum()
    }

    /// Blood volume lost per second across every untreated wound.
    pub fn total_bleed_rate(&self) -> f32 {
        self.wounds.iter().map(|w| w.bleed_rate).sum()
    }

    /// Advances the cardiovascular model by `dt` seconds: applies bleeding
    /// from the current wound list, then lets passive regen claw a little
    /// back. Call once per frame.
    pub fn tick(&mut self, dt: f32) {
        let bleed = self.total_bleed_rate() * dt;
        if bleed > 0.0 {
            self.cardio.apply_drain(bleed);
        }
        self.cardio.regen(dt);
    }

    /// Drains blood volume from a source other than a wound — starvation,
    /// dehydration, anything else that should erode the same consciousness
    /// pipeline instead of needing its own KO/death logic.
    pub fn apply_external_drain(&mut self, amount: f32) {
        self.cardio.apply_drain(amount);
    }

    pub fn blood_volume(&self) -> f32 {
        self.cardio.blood_volume()
    }

    pub fn is_ko(&self) -> bool {
        self.cardio.is_ko()
    }

    pub fn is_dead(&self) -> bool {
        self.cardio.is_dead()
    }

    /// Clears every wound and restores the cardiovascular state to full —
    /// used when the player resets to the starting ledge, so repeated
    /// testing (or dying) doesn't leave old damage lying around.
    pub fn clear(&mut self) {
        self.wounds.clear();
        self.cardio = Cardio::default();
    }
}

#[cfg(test)]
mod tests {
    use super::super::wound::{landing_wound, WoundKind};
    use super::*;

    #[test]
    fn fresh_body_has_no_wounds() {
        let body = BodyState::default();
        assert_eq!(body.wound_count(), 0);
        assert_eq!(body.total_pain(), 0.0);
        assert_eq!(body.total_bleed_rate(), 0.0);
    }

    #[test]
    fn applied_wounds_accumulate_pain_and_bleeding() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::LeftLeg, 0.5).unwrap());
        body.apply_wound(landing_wound(BodyRegion::RightLeg, 0.5).unwrap());
        assert_eq!(body.wound_count(), 2);
        assert!(body.total_pain() > 0.0);
        assert!(body.total_bleed_rate() > 0.0);
        assert_eq!(body.wound_count_in(BodyRegion::LeftLeg), 1);
        assert_eq!(body.wound_count_in(BodyRegion::Head), 0);
    }

    #[test]
    fn clear_removes_every_wound() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::Torso, 0.9).unwrap());
        body.clear();
        assert_eq!(body.wound_count(), 0);
        assert_eq!(body.total_pain(), 0.0);
    }

    #[test]
    fn wound_kind_is_preserved_in_the_list() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::Head, 0.95).unwrap());
        assert_eq!(body.wounds()[0].kind, WoundKind::Fracture);
    }

    #[test]
    fn fresh_body_is_conscious_and_full_blood() {
        let body = BodyState::default();
        assert_eq!(body.blood_volume(), 1.0);
        assert!(!body.is_ko());
        assert!(!body.is_dead());
    }

    #[test]
    fn ticking_with_a_bleeding_wound_drains_blood_over_time() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::LeftLeg, 0.9).unwrap());
        let before = body.blood_volume();
        for _ in 0..60 {
            body.tick(1.0 / 60.0);
        }
        assert!(body.blood_volume() < before);
    }

    #[test]
    fn enough_bleeding_eventually_kills() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::LeftLeg, 1.0).unwrap());
        body.apply_wound(landing_wound(BodyRegion::RightLeg, 1.0).unwrap());
        for _ in 0..600 {
            body.tick(1.0 / 60.0);
        }
        assert!(body.is_dead());
    }

    #[test]
    fn external_drain_can_ko_without_any_wounds() {
        let mut body = BodyState::default();
        body.apply_external_drain(0.7);
        assert!(body.is_ko());
        assert_eq!(body.wound_count(), 0);
    }

    #[test]
    fn clear_restores_full_blood_volume() {
        let mut body = BodyState::default();
        body.apply_external_drain(0.9);
        body.clear();
        assert_eq!(body.blood_volume(), 1.0);
        assert!(!body.is_dead());
    }
}
