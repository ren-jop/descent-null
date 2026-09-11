//! Top-level game state: ties the world, player, and entities together
//! into a single tick/update loop. This is the module a front end
//! (terminal, GUI, whatever) actually drives.

use crate::body::Region;
use crate::crafting::{self, CraftError, RecipeId};
use crate::entities::{attack_target_region, Enemy};
use crate::inventory::ItemId;
use crate::player::Player;
use crate::world::{generate_layer, Layer, ResourceKind, Tile};
use rand::{Rng, SeedableRng};

pub const LAYER_WIDTH: usize = 40;
pub const LAYER_HEIGHT: usize = 24;
const FALL_DAMAGE_THRESHOLD: i32 = 3; // tiles fallen before it starts to hurt
const FALL_DAMAGE_PER_TILE: f32 = 9.0;
const SURVIVAL_TICK_SECONDS: f64 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Playing,
    Extracted,
    Dead,
}

pub struct GameState {
    pub layer: Layer,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub status: Status,
    pub log: Vec<String>,
    pub seed: u64,
    pub depth: u32,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        Self::for_depth(seed, 1)
    }

    fn for_depth(seed: u64, depth: u32) -> Self {
        let ambient_temp = 50.0 - depth as f32 * 3.0; // deeper = colder, in this original setting
        let layer = generate_layer(seed, LAYER_WIDTH, LAYER_HEIGHT, ambient_temp);
        let mut player = Player::new(2, 2);
        player.x = 2;
        player.y = 2;

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0x5151);
        let mut enemies = Vec::new();
        for _ in 0..(2 + depth) {
            let ex = rng.gen_range(6..LAYER_WIDTH as i32 - 2);
            let ey = rng.gen_range(2..LAYER_HEIGHT as i32 - 2);
            if layer.get(ex, ey) != Tile::Wall {
                enemies.push(Enemy::new(ex, ey));
            }
        }

        GameState {
            layer,
            player,
            enemies,
            status: Status::Playing,
            log: vec!["You wake at the edge of the descent. Find the way down alive.".to_string()],
            seed,
            depth,
        }
    }

    pub fn restart(&mut self) {
        let seed: u64 = rand::random();
        *self = GameState::new(seed);
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
        if self.log.len() > 6 {
            self.log.remove(0);
        }
    }

    pub fn last_log(&self) -> &str {
        self.log.last().map(|s| s.as_str()).unwrap_or("")
    }

    /// Attempt to move by (dx, dy). Handles walls, falling, resource
    /// pickup, hazards, and reaching extraction.
    pub fn try_move(&mut self, dx: i32, dy: i32) {
        if self.status != Status::Playing {
            return;
        }
        if self.player.speed_multiplier() < 0.05 {
            self.push_log("Too weak to move.");
            return;
        }

        let nx = self.player.x + dx;
        let ny = self.player.y + dy;
        if self.layer.get(nx, ny) == Tile::Wall {
            self.push_log("Blocked by rock.");
            return;
        }

        // falling: moving down through empty space accumulates fall
        // distance; landing on solid ground (or reversing out of a fall)
        // resolves it into fall damage if it went on long enough
        if dy > 0 {
            self.player.fall_distance += 1;
        }
        let landed_on_ground = self.layer.get(nx, ny + 1) == Tile::Wall;
        if landed_on_ground || dy <= 0 {
            self.resolve_fall();
        }

        self.player.x = nx;
        self.player.y = ny;
        self.player.survival.spend_stamina(0.5);

        match self.layer.get(nx, ny) {
            Tile::Resource(kind) => {
                self.collect_resource(kind);
                self.layer.set(nx as usize, ny as usize, Tile::Empty);
            }
            Tile::Hazard => {
                self.player.body.apply_injury(Region::LeftLeg, 25.0);
                self.push_log("A hidden hazard tears into your leg.");
                self.layer.set(nx as usize, ny as usize, Tile::Empty);
            }
            Tile::Water => {
                self.push_log("Murky water -- probably not safe to drink untreated.");
            }
            Tile::Extraction => {
                self.status = Status::Extracted;
                self.push_log("You reach the extraction point. You're out.");
            }
            _ => {}
        }

        self.check_enemy_contact();
        self.check_death();
    }

    fn resolve_fall(&mut self) {
        if self.player.fall_distance > FALL_DAMAGE_THRESHOLD {
            let excess = (self.player.fall_distance - FALL_DAMAGE_THRESHOLD) as f32;
            let severity = excess * FALL_DAMAGE_PER_TILE;
            let region = if severity > 60.0 { Region::Torso } else { Region::LeftLeg };
            self.player.body.apply_injury(region, severity.min(95.0));
            self.push_log(format!(
                "Hard landing after a {}-tile fall.",
                self.player.fall_distance
            ));
        }
        self.player.fall_distance = 0;
    }

    fn collect_resource(&mut self, kind: ResourceKind) {
        match kind {
            ResourceKind::Food => {
                self.player.inventory.add(ItemId::Ration, 1);
                self.push_log("Found a ration pack.");
            }
            ResourceKind::Water => {
                self.player.inventory.add(ItemId::PurifiedWater, 1);
                self.push_log("Found purified water.");
            }
            ResourceKind::Medical => {
                self.player.inventory.add(ItemId::Bandage, 1);
                self.push_log("Found a bandage.");
            }
            ResourceKind::Material => {
                self.player.inventory.add(ItemId::ScrapMetal, 1);
                self.player.inventory.add(ItemId::Fiber, 1);
                self.push_log("Salvaged some scrap and fiber.");
            }
        }
    }

    fn check_enemy_contact(&mut self) {
        let px = self.player.x;
        let py = self.player.y;
        let mut hits = 0;
        for enemy in &mut self.enemies {
            if !enemy.is_dead() && enemy.adjacent_to(px, py) {
                hits += 1;
            }
        }
        if hits > 0 {
            // deterministic-ish region selection from player position so
            // the same situation is reproducible in tests, without needing
            // a stored RNG on GameState
            let roll = ((px * 7 + py * 13) % 600) as f32 / 600.0;
            let region = attack_target_region(roll);
            self.player.body.apply_injury(region, 22.0 * hits as f32);
            self.push_log("Something attacks you in the dark.");
        }
    }

    pub fn use_medical_item(&mut self, item: ItemId, region: Region) -> bool {
        if self.player.inventory.remove(item, 1) == 0 {
            return false;
        }
        match item {
            ItemId::Bandage => self.player.body.apply_bandage(region),
            ItemId::Splint => self.player.body.apply_splint(region),
            ItemId::Antiseptic => self.player.body.apply_antiseptic(region),
            ItemId::ClottingAgent => self.player.body.apply_clotting_agent(region),
            ItemId::Painkiller => self.player.body.apply_painkiller(region),
            _ => {
                // not a medical item -- refund it, this call was a mistake
                self.player.inventory.add(item, 1);
                return false;
            }
        }
        self.push_log("Treated an injury.");
        true
    }

    pub fn eat_ration(&mut self) -> bool {
        if self.player.inventory.remove(ItemId::Ration, 1) == 0 {
            return false;
        }
        self.player.survival.eat(30.0);
        self.push_log("Ate a ration pack.");
        true
    }

    pub fn drink_water(&mut self) -> bool {
        if self.player.inventory.remove(ItemId::PurifiedWater, 1) == 0 {
            return false;
        }
        self.player.survival.drink(35.0);
        self.push_log("Drank purified water.");
        true
    }

    pub fn craft(&mut self, recipe_id: RecipeId) -> Result<(), CraftError> {
        let result = crafting::craft(
            &mut self.player.inventory,
            recipe_id,
            self.player.attributes.intelligence,
        );
        match &result {
            Ok(()) => self.push_log("Crafted something useful."),
            Err(CraftError::MissingIngredients) => self.push_log("Missing ingredients."),
            Err(CraftError::IntelligenceTooLow) => {
                self.push_log("You don't understand how to make that yet.")
            }
            Err(CraftError::InventoryFull) => self.push_log("No room to carry the result."),
        }
        result
    }

    /// Advance survival stats, body simulation, and enemy AI by one
    /// real-time tick (called on a fixed cadence by the front end).
    pub fn tick(&mut self) {
        if self.status != Status::Playing {
            return;
        }
        self.player.survival.tick(self.layer.ambient_temp);
        self.player.survival.apply_to_body(&mut self.player.body);
        self.player.body.tick();

        let px = self.player.x;
        let py = self.player.y;
        let is_wall = |x: i32, y: i32| self.layer.get(x, y) == Tile::Wall;
        for enemy in &mut self.enemies {
            if !enemy.is_dead() {
                enemy.step(px, py, is_wall);
            }
        }
        self.check_enemy_contact();
        self.check_death();
    }

    fn check_death(&mut self) {
        if self.player.body.dead && self.status == Status::Playing {
            self.status = Status::Dead;
            let cause = self
                .player
                .body
                .cause_of_death
                .clone()
                .unwrap_or_else(|| "unknown causes".to_string());
            self.push_log(format!("You didn't make it: {cause}."));
        }
    }
}

pub const TICK_SECONDS: f64 = SURVIVAL_TICK_SECONDS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_in_playing_state() {
        let g = GameState::new(1);
        assert_eq!(g.status, Status::Playing);
        assert!(!g.player.is_dead());
    }

    #[test]
    fn walking_into_a_wall_does_not_move_the_player() {
        let mut g = GameState::new(1);
        // find a direction that's actually a wall next to spawn, or trust
        // spawn area is kept clear and instead assert a genuinely walled
        // offscreen direction never moves the player out of bounds
        let (x, y) = (g.player.x, g.player.y);
        g.try_move(-100, 0); // absurd delta; get() treats out-of-bounds as Wall
        assert_eq!((g.player.x, g.player.y), (x, y));
    }

    #[test]
    fn falling_several_tiles_then_landing_causes_injury() {
        let mut g = GameState::new(2);
        // carve a clear vertical shaft with a solid floor at the bottom so
        // there's an actual landing to test, rather than infinite open air
        for y in 0..14 {
            g.layer.set(5, y, Tile::Empty);
        }
        g.layer.set(5, 14, Tile::Wall);
        g.player.x = 5;
        g.player.y = 0;
        let condition_before = g.player.body.region(Region::LeftLeg).condition
            + g.player.body.region(Region::Torso).condition;
        for _ in 0..13 {
            g.try_move(0, 1);
        }
        let condition_after = g.player.body.region(Region::LeftLeg).condition
            + g.player.body.region(Region::Torso).condition;
        assert!(
            condition_after < condition_before,
            "landing after a long fall should have caused some injury"
        );
    }

    #[test]
    fn eating_a_ration_restores_hunger_and_consumes_it() {
        let mut g = GameState::new(3);
        g.player.inventory.add(ItemId::Ration, 1);
        for _ in 0..50 {
            g.tick();
        }
        let hunger_before = g.player.survival.hunger;
        assert!(g.eat_ration());
        assert!(g.player.survival.hunger > hunger_before);
        assert_eq!(g.player.inventory.quantity_of(ItemId::Ration), 0);
    }

    #[test]
    fn bandaging_reduces_bleeding_and_consumes_the_bandage() {
        let mut g = GameState::new(4);
        g.player.inventory.add(ItemId::Bandage, 1);
        g.player.body.apply_injury(Region::Torso, 60.0);
        let bleeding_before = g.player.body.region(Region::Torso).bleeding;
        assert!(g.use_medical_item(ItemId::Bandage, Region::Torso));
        assert!(g.player.body.region(Region::Torso).bleeding < bleeding_before);
        assert_eq!(g.player.inventory.quantity_of(ItemId::Bandage), 0);
    }

    #[test]
    fn using_a_medical_item_you_do_not_have_fails_cleanly() {
        let mut g = GameState::new(5);
        assert!(!g.use_medical_item(ItemId::Bandage, Region::Torso));
    }

    #[test]
    fn severe_neglect_eventually_ends_the_game() {
        let mut g = GameState::new(6);
        g.player.body.apply_injury(Region::Torso, 95.0);
        g.player.body.apply_injury(Region::Head, 95.0);
        let mut ticks = 0;
        while g.status == Status::Playing && ticks < 2000 {
            g.tick();
            ticks += 1;
        }
        assert_eq!(g.status, Status::Dead);
    }

    #[test]
    fn reaching_extraction_tile_wins_the_run() {
        let mut g = GameState::new(9);
        // force a known layout: clear corridor straight to a placed
        // extraction tile so this test doesn't depend on generated terrain
        for x in 0..10 {
            g.layer.set(x, 2, Tile::Empty);
        }
        g.layer.set(9, 2, Tile::Extraction);
        g.player.x = 0;
        g.player.y = 2;
        for _ in 0..9 {
            g.try_move(1, 0);
        }
        assert_eq!(g.status, Status::Extracted);
    }
}
