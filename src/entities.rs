//! Simple original enemy: a territorial creature that senses the player
//! within a radius, chases, and injures on contact. Not a copy of any
//! existing game's creature -- generic patrol/chase/attack behaviour.

use crate::body::Region;

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub home_x: i32,
    pub home_y: i32,
    pub sense_radius: i32,
    pub aware: bool,
    pub health: f32,
}

impl Enemy {
    pub fn new(x: i32, y: i32) -> Self {
        Enemy {
            x,
            y,
            home_x: x,
            home_y: y,
            sense_radius: 5,
            aware: false,
            health: 40.0,
        }
    }

    /// One AI step. `is_wall` lets the caller supply the current layer's
    /// collision check without this module depending on `world::Layer`
    /// directly, so entity logic stays independently testable.
    pub fn step(&mut self, player_x: i32, player_y: i32, is_wall: impl Fn(i32, i32) -> bool) {
        let dist = (self.x - player_x).abs() + (self.y - player_y).abs();
        if dist <= self.sense_radius {
            self.aware = true;
        }
        if self.aware && dist > self.sense_radius * 3 {
            self.aware = false; // lost the player, will drift home
        }

        let (target_x, target_y) = if self.aware {
            (player_x, player_y)
        } else {
            (self.home_x, self.home_y)
        };

        let dx = (target_x - self.x).signum();
        let dy = (target_y - self.y).signum();

        // try the more urgent axis first, fall back to the other if blocked
        if dx != 0 && !is_wall(self.x + dx, self.y) {
            self.x += dx;
        } else if dy != 0 && !is_wall(self.x, self.y + dy) {
            self.y += dy;
        }
    }

    pub fn adjacent_to(&self, x: i32, y: i32) -> bool {
        (self.x - x).abs() <= 1 && (self.y - y).abs() <= 1
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }
}

/// Which body region a contact attack lands on -- deterministic from a
/// caller-supplied roll so behaviour stays testable, rather than reaching
/// into a global RNG from inside this module.
pub fn attack_target_region(roll: f32) -> Region {
    match (roll.clamp(0.0, 0.999) * 6.0) as u32 {
        0 => Region::Head,
        1 => Region::Torso,
        2 => Region::LeftArm,
        3 => Region::RightArm,
        4 => Region::LeftLeg,
        _ => Region::RightLeg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_ignores_distant_player() {
        let mut e = Enemy::new(0, 0);
        e.step(20, 20, |_, _| false);
        assert!(!e.aware);
        assert_eq!((e.x, e.y), (0, 0)); // home == start, nothing to chase toward
    }

    #[test]
    fn enemy_chases_player_within_sense_radius() {
        let mut e = Enemy::new(0, 0);
        e.step(2, 0, |_, _| false);
        assert!(e.aware);
        assert_eq!(e.x, 1); // stepped one tile toward the player
    }

    #[test]
    fn enemy_does_not_walk_into_walls() {
        let mut e = Enemy::new(0, 0);
        // wall directly toward the player on the x axis; should try y instead (dy=0 here, so it just doesn't move)
        e.step(2, 0, |x, y| x == 1 && y == 0);
        assert_eq!(e.x, 0, "should not have moved into the wall");
    }

    #[test]
    fn attack_region_mapping_covers_all_regions_without_panicking() {
        for i in 0..600 {
            let roll = i as f32 / 600.0;
            let _ = attack_target_region(roll); // must not panic across the full [0,1) range
        }
    }
}
