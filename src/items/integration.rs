//! bevy wiring for items: the player's inventory, world pickups, and the
//! proximity-based collection system.

use bevy::prelude::*;

use crate::body::Body;
use crate::player::Player;
use crate::survival::Survival;

use super::craft;
use super::inventory::Inventory;
use super::item::{ItemKind, ItemStack};

// default carry-weight cap. the doc calls for a real "can't carry
// everything" tradeoff, so this is deliberately tight, not generous.
const DEFAULT_CAPACITY: u32 = 12;
const PICKUP_RADIUS: f32 = 36.0;
const EVENT_DISPLAY_SECONDS: f32 = 3.0;

impl Default for Inventory {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[derive(Component, Default)]
pub struct PlayerInventory(pub Inventory);

#[derive(Component)]
pub struct Pickup(pub ItemStack);

/// a brief contextual notification — pickups, item use, crafting, combat.
/// shown as a fading toast, not a permanent HUD line (see ui::update_event_toast).
#[derive(Resource, Default)]
pub struct LastEvent {
    pub text: String,
    pub remaining: f32,
}

impl LastEvent {
    pub fn show(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.remaining = EVENT_DISPLAY_SECONDS;
    }
}

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastEvent>()
            .add_systems(Update, (collect_pickups, use_supplies, craft_item, tick_last_event));
    }
}

fn tick_last_event(time: Res<Time>, mut last: ResMut<LastEvent>) {
    if last.remaining > 0.0 {
        last.remaining = (last.remaining - time.delta_secs()).max(0.0);
    }
}

fn collect_pickups(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    pickups: Query<(Entity, &Transform, &Pickup)>,
    mut inventory: Query<&mut PlayerInventory>,
    mut last: ResMut<LastEvent>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut inventory) = inventory.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    for (entity, transform, pickup) in &pickups {
        if player_pos.distance(transform.translation.truncate()) > PICKUP_RADIUS {
            continue;
        }
        if inventory.0.add(pickup.0) {
            last.show(format!("picked up {} {}", pickup.0.quantity, pickup.0.kind.label()));
            commands.entity(entity).despawn();
        } else {
            last.show(format!("inventory full — can't carry {}", pickup.0.kind.label()));
        }
    }
}

/// `F` — use the single most relevant supply for the player's current
/// problem: eat/drink if low, bandage active bleeding, splint a fracture,
/// or medkit if hurt at all. one action, no item-picker menu yet.
fn use_supplies(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerInventory, &mut Body, &mut Survival)>,
    mut last: ResMut<LastEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok((mut inventory, mut body, mut survival)) = query.single_mut() else {
        return;
    };
    if survival.0.hunger() < 0.5 && inventory.0.remove(ItemKind::Food, 1) > 0 {
        survival.0.eat(0.4);
        last.show("ate food");
    } else if survival.0.thirst() < 0.5 && inventory.0.remove(ItemKind::Water, 1) > 0 {
        survival.0.drink(0.4);
        last.show("drank water");
    } else if body.0.total_bleed_rate() > 0.0 && inventory.0.remove(ItemKind::Bandage, 1) > 0 {
        body.0.treat_worst_bleeding();
        last.show("used bandage");
    } else if body.0.has_untreated_leg_fracture() && inventory.0.remove(ItemKind::Splint, 1) > 0 {
        body.0.treat_fracture();
        last.show("used splint");
    } else if (body.0.blood_volume() < 0.9 || body.0.wound_count() > 0)
        && inventory.0.remove(ItemKind::Medkit, 1) > 0
    {
        body.0.heal_blood_volume(0.35);
        body.0.treat_pain();
        last.show("used medkit");
    } else {
        last.show("nothing useful to use right now");
    }
}

/// `C` — craft the first recipe you can afford. see items::craft.
fn craft_item(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerInventory>,
    mut last: ResMut<LastEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyC) {
        return;
    }
    let Ok(mut inventory) = query.single_mut() else {
        return;
    };
    match craft::try_craft(&mut inventory.0) {
        Some(kind) => last.show(format!("crafted {}", kind.label())),
        None => last.show("not enough materials to craft anything"),
    }
}
