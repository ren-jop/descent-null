//! Wounds: the atomic unit of injury. A landing, a bite, or a blade all
//! ultimately produce a `Wound`; the body sim only ever reasons about the
//! accumulated list (see `BodyState`), never a per-region float.

use super::region::BodyRegion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WoundKind {
    Bruise,
    Laceration,
    Fracture,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wound {
    pub region: BodyRegion,
    pub kind: WoundKind,
    /// Ongoing pain contribution while this wound is untreated.
    pub pain: f32,
    /// Blood volume lost per second while untreated. Zero for wounds that
    /// don't bleed (a plain bruise).
    pub bleed_rate: f32,
    /// Bandaged (stops bleeding) or splinted (removes the fracture
    /// movement penalty). See `BodyState::treat_*`.
    pub treated: bool,
}

/// Severity (from `physics::landing_severity`, 0..=1) at which a hard
/// landing starts splitting skin instead of just bruising.
pub const LACERATION_SEVERITY: f32 = 0.45;
/// Severity at which a hard landing breaks bone.
pub const FRACTURE_SEVERITY: f32 = 0.85;

const PAIN_PER_SEVERITY: f32 = 6.0;
// Untreated bleeding should feel urgent enough that the player notices the
// health bar moving and reacts to the FIELD LOG, while still leaving time to
// select/use a bandage. Fractures remain more dangerous than lacerations.
const LACERATION_BLEED_PER_SEVERITY: f32 = 0.032;
const FRACTURE_BLEED_PER_SEVERITY: f32 = 0.062;

/// Turns a landing-impact severity (0..=1, see `physics::landing_severity`)
/// into a wound on the given region. A severity of 0 or below produces no
/// wound — callers already filter through `landing_severity`, which returns
/// `None` below the hard-landing threshold, so this is mostly a safety net.
pub fn landing_wound(region: BodyRegion, severity: f32) -> Option<Wound> {
    if severity <= 0.0 {
        return None;
    }
    let severity = severity.clamp(0.0, 1.0);
    let kind = if severity >= FRACTURE_SEVERITY {
        WoundKind::Fracture
    } else if severity >= LACERATION_SEVERITY {
        WoundKind::Laceration
    } else {
        WoundKind::Bruise
    };
    let bleed_rate = match kind {
        WoundKind::Fracture => severity * FRACTURE_BLEED_PER_SEVERITY,
        WoundKind::Laceration => severity * LACERATION_BLEED_PER_SEVERITY,
        WoundKind::Bruise => 0.0,
    };
    Some(Wound {
        region,
        kind,
        pain: severity * PAIN_PER_SEVERITY,
        bleed_rate,
        treated: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mild_impact_is_a_bruise_with_no_bleeding() {
        let wound = landing_wound(BodyRegion::LeftLeg, 0.1).unwrap();
        assert_eq!(wound.kind, WoundKind::Bruise);
        assert_eq!(wound.bleed_rate, 0.0);
        assert!(wound.pain > 0.0);
    }

    #[test]
    fn moderate_impact_lacerates_and_bleeds() {
        let wound = landing_wound(BodyRegion::RightLeg, 0.6).unwrap();
        assert_eq!(wound.kind, WoundKind::Laceration);
        assert!(wound.bleed_rate > 0.0);
    }

    #[test]
    fn severe_impact_fractures() {
        let wound = landing_wound(BodyRegion::LeftLeg, 0.95).unwrap();
        assert_eq!(wound.kind, WoundKind::Fracture);
        assert!(wound.bleed_rate > 0.0);
    }

    #[test]
    fn worse_impacts_hurt_and_bleed_more() {
        let mild = landing_wound(BodyRegion::Torso, 0.5).unwrap();
        let severe = landing_wound(BodyRegion::Torso, 0.9).unwrap();
        assert!(severe.pain > mild.pain);
        assert!(severe.bleed_rate > mild.bleed_rate);
    }

    #[test]
    fn zero_severity_produces_no_wound() {
        assert!(landing_wound(BodyRegion::Head, 0.0).is_none());
    }

    #[test]
    fn severity_above_one_is_clamped_not_extrapolated() {
        let at_max = landing_wound(BodyRegion::Head, 1.0).unwrap();
        let over_max = landing_wound(BodyRegion::Head, 5.0).unwrap();
        assert_eq!(at_max.pain, over_max.pain);
        assert_eq!(at_max.bleed_rate, over_max.bleed_rate);
    }
}
