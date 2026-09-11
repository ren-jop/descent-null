//! Landing impact from downward speed. Milestone 1 records the event;
//! Milestone 2 will turn it into location-specific wounds.

/// Downward speed (world units / second) below which a landing is a step, not a hit.
pub const LANDING_SPEED_THRESHOLD: f32 = 450.0;
/// Extra speed that maps to a full-severity (1.0) impact.
pub const LANDING_FULL_SEVERITY_SPAN: f32 = 550.0;

/// Tracks the worst downward speed while airborne so a landing can use
/// actual impact velocity instead of fall *tiles*.
#[derive(Clone, Debug, Default)]
pub struct FallTracker {
    max_downward_speed: f32,
    airborne: bool,
}

impl FallTracker {
    pub fn observe(&mut self, grounded: bool, velocity_y: f32) -> Option<f32> {
        if grounded {
            if self.airborne {
                let speed = self.max_downward_speed;
                self.max_downward_speed = 0.0;
                self.airborne = false;
                return Some(speed);
            }
            self.max_downward_speed = 0.0;
            None
        } else {
            self.airborne = true;
            if velocity_y < 0.0 {
                self.max_downward_speed = self.max_downward_speed.max(-velocity_y);
            }
            None
        }
    }
}

/// 0..=1 impact factor from landing speed. `None` means the landing is
/// harmless for simulation purposes.
pub fn landing_severity(downward_speed: f32) -> Option<f32> {
    if downward_speed < LANDING_SPEED_THRESHOLD {
        None
    } else {
        let t = (downward_speed - LANDING_SPEED_THRESHOLD) / LANDING_FULL_SEVERITY_SPAN;
        Some(t.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gentle_touchdown_is_not_an_impact() {
        assert_eq!(landing_severity(200.0), None);
        assert_eq!(landing_severity(LANDING_SPEED_THRESHOLD - 1.0), None);
    }

    #[test]
    fn severity_scales_with_speed() {
        let a = landing_severity(LANDING_SPEED_THRESHOLD + 100.0).unwrap();
        let b = landing_severity(LANDING_SPEED_THRESHOLD + 400.0).unwrap();
        assert!(b > a);
        assert!(a > 0.0 && b <= 1.0);
    }

    #[test]
    fn fall_tracker_reports_peak_downward_speed_on_land() {
        let mut t = FallTracker::default();
        assert!(t.observe(false, -100.0).is_none());
        assert!(t.observe(false, -800.0).is_none());
        assert!(t.observe(false, -300.0).is_none());
        let landed = t.observe(true, 0.0).unwrap();
        assert!((landed - 800.0).abs() < f32::EPSILON);
        assert!(t.observe(true, 0.0).is_none(), "staying grounded is not a new landing");
    }
}
