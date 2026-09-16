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

    pub fn total_pain(&self) -> f32 {
        self.wounds.iter().map(|w| w.pain).sum()
    }

    pub fn total_bleed_rate(&self) -> f32 {
        self.wounds.iter().map(|w| w.bleed_rate).sum()
    }

    pub fn tick(&mut self, dt: f32) {
        let bleed = self.total_bleed_rate() * dt;
        if bleed > 0.0 {
            self.cardio.apply_drain(bleed);
        }
        self.cardio.regen(dt);
    }

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

    pub fn heal_blood_volume(&mut self, amount: f32) {
        self.cardio.heal(amount);
    }

    /// Stops every currently active bleed. This is deliberately forgiving:
    /// one bandage use should feel reliable instead of leaving an invisible
    /// second bleed running in the background.
    pub fn treat_all_bleeding(&mut self) -> usize {
        let mut treated = 0;
        for wound in &mut self.wounds {
            if wound.bleed_rate > 0.0 {
                wound.bleed_rate = 0.0;
                wound.treated = true;
                treated += 1;
            }
        }
        treated
    }

    /// Compatibility helper for older callers/tests.
    pub fn treat_worst_bleeding(&mut self) -> bool {
        self.treat_all_bleeding() > 0
    }

    pub fn has_untreated_leg_fracture(&self) -> bool {
        self.wounds.iter().any(|w| {
            w.kind == super::wound::WoundKind::Fracture
                && matches!(w.region, BodyRegion::LeftLeg | BodyRegion::RightLeg)
                && !w.treated
        })
    }

    pub fn treat_fracture(&mut self) -> bool {
        let target = self
            .wounds
            .iter_mut()
            .find(|w| w.kind == super::wound::WoundKind::Fracture && !w.treated);
        match target {
            Some(wound) => {
                wound.treated = true;
                true
            }
            None => false,
        }
    }

    pub fn treat_pain(&mut self) -> bool {
        let target = self.wounds.iter_mut().find(|w| !w.treated && w.pain > 0.0);
        match target {
            Some(wound) => {
                wound.pain = 0.0;
                wound.treated = true;
                true
            }
            None => false,
        }
    }

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

    #[test]
    fn bandaging_stops_all_active_bleeding() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::LeftLeg, 0.9).unwrap());
        body.apply_wound(landing_wound(BodyRegion::RightLeg, 0.9).unwrap());
        assert!(body.total_bleed_rate() > 0.0);
        assert_eq!(body.treat_all_bleeding(), 2);
        assert_eq!(body.total_bleed_rate(), 0.0);
    }

    #[test]
    fn bandaging_with_nothing_bleeding_does_nothing() {
        let mut body = BodyState::default();
        assert!(!body.treat_worst_bleeding());
    }

    #[test]
    fn splinting_clears_the_fracture_penalty_flag() {
        let mut body = BodyState::default();
        body.apply_wound(landing_wound(BodyRegion::LeftLeg, 0.95).unwrap());
        assert!(body.has_untreated_leg_fracture());
        assert!(body.treat_fracture());
        assert!(!body.has_untreated_leg_fracture());
    }

    #[test]
    fn medkit_heals_blood_volume() {
        let mut body = BodyState::default();
        body.apply_external_drain(0.5);
        body.heal_blood_volume(0.3);
        assert!((body.blood_volume() - 0.8).abs() < 0.001);
    }
}
