//! enemy stats and the idle/chase/attack decision — pure logic, no bevy.
//! deliberately a flat 3-state machine, not a behavior tree: this is
//! meant to be one simple, reliable enemy, not a framework.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyState {
    Idle,
    Chasing,
    Attacking,
}

/// beyond this distance the enemy doesn't notice the player at all.
pub const DETECTION_RANGE: f32 = 260.0;
/// within this distance it stops closing in and attacks instead.
pub const ATTACK_RANGE: f32 = 34.0;

/// picks a state purely from distance to the player. no memory/hysteresis
/// on purpose — simple and reliable beats a fancier state machine here.
pub fn state_for_distance(distance: f32) -> EnemyState {
    if distance <= ATTACK_RANGE {
        EnemyState::Attacking
    } else if distance <= DETECTION_RANGE {
        EnemyState::Chasing
    } else {
        EnemyState::Idle
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyStats {
    health: f32,
    max_health: f32,
}

impl EnemyStats {
    pub fn new(max_health: f32) -> Self {
        Self { health: max_health, max_health }
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn max_health(&self) -> f32 {
        self.max_health
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_away_is_idle() {
        assert_eq!(state_for_distance(1000.0), EnemyState::Idle);
    }

    #[test]
    fn within_detection_is_chasing() {
        assert_eq!(state_for_distance(150.0), EnemyState::Chasing);
    }

    #[test]
    fn within_attack_range_is_attacking() {
        assert_eq!(state_for_distance(10.0), EnemyState::Attacking);
    }

    #[test]
    fn damage_reduces_health_and_clamps_at_zero() {
        let mut stats = EnemyStats::new(10.0);
        stats.take_damage(4.0);
        assert_eq!(stats.health(), 6.0);
        assert!(!stats.is_dead());
        stats.take_damage(100.0);
        assert_eq!(stats.health(), 0.0);
        assert!(stats.is_dead());
    }
}
