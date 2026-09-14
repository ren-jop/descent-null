//! `BodyState`: the running list of wounds for one body. Engine-agnostic —
//! wrapped in a Bevy `Component` in `integration.rs` so this stays testable
//! without spinning up an `App`.

use super::region::BodyRegion;
use super::wound::Wound;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BodyState {
    wounds: Vec<Wound>,
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

    /// Blood volume lost per second across every untreated wound. There is
    /// no cardiovascular model consuming this yet (milestone 2, later
    /// slice) — for now it's informational, surfaced on the HUD.
    pub fn total_bleed_rate(&self) -> f32 {
        self.wounds.iter().map(|w| w.bleed_rate).sum()
    }

    /// Clears every wound — used when the player resets to the starting
    /// ledge, so repeated testing doesn't accumulate injuries forever.
    pub fn clear(&mut self) {
        self.wounds.clear();
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
}
