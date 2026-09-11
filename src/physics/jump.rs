//! Coyote time and jump buffering.
//!
//! These exist so a heavy, slightly late jump still works, without turning
//! movement into an arcade hop-fest. A jump is consumed only when both a
//! recent ground contact (coyote) and a recent jump intent (buffer) exist.

/// Seconds after leaving ground during which a jump is still legal.
pub const JUMP_COYOTE: f32 = 0.12;
/// Seconds a jump press is remembered while airborne.
pub const JUMP_BUFFER: f32 = 0.12;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JumpAssist {
    coyote: f32,
    buffer: f32,
}

impl JumpAssist {
    pub fn tick(&mut self, dt: f32, grounded: bool) {
        if grounded {
            self.coyote = JUMP_COYOTE;
        } else {
            self.coyote = (self.coyote - dt).max(0.0);
        }
        self.buffer = (self.buffer - dt).max(0.0);
    }

    pub fn press_jump(&mut self) {
        self.buffer = JUMP_BUFFER;
    }

    pub fn can_jump(&self) -> bool {
        self.coyote > 0.0 && self.buffer > 0.0
    }

    /// Returns true if a jump should fire this frame. Consumes both windows
    /// so a single press cannot produce two takeoffs.
    pub fn consume_jump(&mut self) -> bool {
        if self.can_jump() {
            self.coyote = 0.0;
            self.buffer = 0.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_jump_fires_immediately() {
        let mut j = JumpAssist::default();
        j.tick(0.016, true);
        j.press_jump();
        assert!(j.consume_jump());
        assert!(!j.consume_jump(), "must not double-jump from one press");
    }

    #[test]
    fn coyote_allows_jump_shortly_after_leaving_ground() {
        let mut j = JumpAssist::default();
        j.tick(0.016, true);
        j.tick(0.05, false);
        j.press_jump();
        assert!(j.consume_jump());
    }

    #[test]
    fn coyote_expires() {
        let mut j = JumpAssist::default();
        j.tick(0.016, true);
        j.tick(JUMP_COYOTE + 0.05, false);
        j.press_jump();
        assert!(!j.consume_jump());
    }

    #[test]
    fn buffer_lets_early_press_fire_on_landing() {
        let mut j = JumpAssist::default();
        j.tick(0.016, false);
        j.press_jump();
        j.tick(0.05, false);
        j.tick(0.016, true);
        assert!(j.consume_jump());
    }

    #[test]
    fn expired_buffer_does_not_jump() {
        let mut j = JumpAssist::default();
        j.press_jump();
        j.tick(JUMP_BUFFER + 0.05, false);
        j.tick(0.016, true);
        assert!(!j.consume_jump());
    }
}
