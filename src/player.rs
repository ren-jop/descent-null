//! Player state: position, attributes, and the body/survival/inventory
//! subsystems combined.

use crate::body::BodyState;
use crate::inventory::Inventory;
use crate::survival::SurvivalState;

#[derive(Clone, Copy, Debug)]
pub struct Attributes {
    pub strength: f32,
    pub resilience: f32,
    pub intelligence: f32,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            strength: 10.0,
            resilience: 10.0,
            intelligence: 10.0,
        }
    }
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub body: BodyState,
    pub survival: SurvivalState,
    pub inventory: Inventory,
    pub attributes: Attributes,
    /// tracks how many consecutive tiles the player has fallen without
    /// landing, so a landing can apply fall damage proportional to drop
    /// distance rather than an instant flat penalty
    pub fall_distance: i32,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        let attributes = Attributes::new();
        Player {
            x,
            y,
            body: BodyState::new(),
            survival: SurvivalState::new(),
            inventory: Inventory::new(20.0 + attributes.strength * 1.5),
            attributes,
            fall_distance: 0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.body.dead
    }

    /// Combined movement multiplier from both body condition and stamina.
    pub fn speed_multiplier(&self) -> f32 {
        (1.0 - self.body.movement_penalty()) * self.survival.stamina_speed_multiplier()
    }
}
