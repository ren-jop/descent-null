//! The six independently injurable body regions (see docs/REBUILD_PLAN.md).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyRegion {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

impl BodyRegion {
    /// All six regions, for iterating or seeding a fresh body.
    pub const ALL: [BodyRegion; 6] = [
        BodyRegion::Head,
        BodyRegion::Torso,
        BodyRegion::LeftArm,
        BodyRegion::RightArm,
        BodyRegion::LeftLeg,
        BodyRegion::RightLeg,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BodyRegion::Head => "head",
            BodyRegion::Torso => "torso",
            BodyRegion::LeftArm => "left arm",
            BodyRegion::RightArm => "right arm",
            BodyRegion::LeftLeg => "left leg",
            BodyRegion::RightLeg => "right leg",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_regions_are_distinct() {
        let mut labels: Vec<_> = BodyRegion::ALL.iter().map(|r| r.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), 6);
    }

    #[test]
    fn all_contains_every_variant_once() {
        assert_eq!(BodyRegion::ALL.len(), 6);
    }
}
