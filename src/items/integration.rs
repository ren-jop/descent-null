//! Bevy wiring for inventory, pickups, selected-item use and crafting.
//! Beginner rule: every pickup says what it does; every number-key selection
//! says exactly what slot/item is active and what F will do.

use bevy::prelude::*;

use crate::body::Body;
use crate::player::Player;
use crate::survival::Survival;

use super::craft;
use super::inventory::Inventory;
use super::item::{ItemKind, ItemStack};

const DEFAULT_CAPACITY: u32 = 12;
const PICKUP_RADIUS: f32 = 40.0;
const EVENT_DISPLAY_SECONDS: f32 = 4.0;

impl Default for Inventory {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[derive(Component, Default)]
pub struct PlayerInventory(pub Inventory);

#[derive(Component)]
pub struct Pickup(pub ItemStack);

#[derive(Resource, Default)]
pub struct SelectedSlot(pub usize);

#[derive(Resource, Default)]
pub struct CraftingMenu {
    pub open: bool,
}

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
            .init_resource::<SelectedSlot>()
            .init_resource::<CraftingMenu>()
            .add_systems(
                Update,
                (
                    collect_pickups,
                    inventory_controls,
                    use_selected_item,
                    crafting_controls,
                    tick_last_event,
                )
                    .chain(),
            );
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
    mut selected: ResMut<SelectedSlot>,
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
            // Select the newly created stack when possible, so a beginner can
            // immediately see it highlighted instead of wondering where it went.
            if let Some(index) = inventory
                .0
                .stacks()
                .iter()
                .position(|stack| stack.kind == pickup.0.kind)
            {
                selected.0 = index.min(8);
            }
            last.show(format!(
                "PICKED UP: {} x{} - {}",
                pickup.0.kind.label().to_uppercase(),
                pickup.0.quantity,
                pickup.0.kind.purpose()
            ));
            commands.entity(entity).despawn();
        } else {
            last.show(format!(
                "PACK FULL: cannot carry {} - use/drop supplies first",
                pickup.0.kind.label().to_uppercase()
            ));
        }
    }
}

fn digit_pressed(keyboard: &ButtonInput<KeyCode>) -> Option<usize> {
    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    keys.iter().position(|key| keyboard.just_pressed(*key))
}

fn inventory_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    crafting: Res<CraftingMenu>,
    mut selected: ResMut<SelectedSlot>,
    inventory: Query<&PlayerInventory>,
    mut last: ResMut<LastEvent>,
) {
    if crafting.open {
        return;
    }
    let Some(slot) = digit_pressed(&keyboard) else {
        return;
    };
    selected.0 = slot;
    if let Ok(inventory) = inventory.single() {
        if let Some(stack) = inventory.0.stacks().get(slot) {
            let action = if stack.kind.is_directly_usable() {
                "Press F to use"
            } else {
                "Press C to craft with it"
            };
            last.show(format!(
                "SLOT {} SELECTED: {} x{} - {}. {}",
                slot + 1,
                stack.kind.label().to_uppercase(),
                stack.quantity,
                stack.kind.purpose(),
                action
            ));
        } else {
            last.show(format!("SLOT {} SELECTED: empty", slot + 1));
        }
    }
}

fn use_selected_item(
    keyboard: Res<ButtonInput<KeyCode>>,
    crafting: Res<CraftingMenu>,
    selected: Res<SelectedSlot>,
    mut query: Query<(&mut PlayerInventory, &mut Body, &mut Survival)>,
    mut last: ResMut<LastEvent>,
) {
    if crafting.open || !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok((mut inventory, mut body, mut survival)) = query.single_mut() else {
        return;
    };
    let Some(stack) = inventory.0.stacks().get(selected.0).copied() else {
        last.show(format!("SLOT {} IS EMPTY - pick up an item first", selected.0 + 1));
        return;
    };

    match stack.kind {
        ItemKind::Food => {
            inventory.0.remove(ItemKind::Food, 1);
            survival.0.eat(0.4);
            last.show("USED FOOD - hunger restored");
        }
        ItemKind::Water => {
            inventory.0.remove(ItemKind::Water, 1);
            survival.0.drink(0.4);
            last.show("USED WATER - thirst restored");
        }
        ItemKind::Bandage => {
            if body.0.total_bleed_rate() <= 0.0 {
                last.show("BANDAGE NOT NEEDED - you are not bleeding");
            } else {
                inventory.0.remove(ItemKind::Bandage, 1);
                body.0.treat_worst_bleeding();
                last.show("USED BANDAGE - worst active bleed stopped");
            }
        }
        ItemKind::Splint => {
            if !body.0.has_untreated_leg_fracture() {
                last.show("SPLINT NOT NEEDED - no untreated leg fracture");
            } else {
                inventory.0.remove(ItemKind::Splint, 1);
                body.0.treat_fracture();
                last.show("USED SPLINT - fracture stabilised");
            }
        }
        ItemKind::Medkit => {
            if body.0.blood_volume() >= 0.99 && body.0.wound_count() == 0 {
                last.show("MEDKIT NOT NEEDED - you are healthy");
            } else {
                inventory.0.remove(ItemKind::Medkit, 1);
                body.0.heal_blood_volume(0.35);
                body.0.treat_pain();
                last.show("USED MEDKIT - blood restored and pain treated");
            }
        }
        ItemKind::Scrap | ItemKind::Cloth | ItemKind::Metal | ItemKind::Battery => {
            last.show(format!(
                "{} IS A CRAFTING MATERIAL - press C to see what it makes",
                stack.kind.label().to_uppercase()
            ));
        }
    }
}

fn crafting_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut crafting: ResMut<CraftingMenu>,
    mut inventory: Query<&mut PlayerInventory>,
    mut last: ResMut<LastEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        crafting.open = false;
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.open = !crafting.open;
        if crafting.open {
            last.show("CRAFTING OPEN - recipes 1, 2 and 3 show exactly what you need");
        }
        return;
    }
    if !crafting.open {
        return;
    }
    let Some(index) = digit_pressed(&keyboard) else {
        return;
    };
    if index >= craft::recipe_count() {
        return;
    }
    let Ok(mut inventory) = inventory.single_mut() else {
        return;
    };
    if !craft::can_craft(&inventory.0, index) {
        last.show(format!("RECIPE {}: missing required materials", index + 1));
        return;
    }
    match craft::try_craft_index(&mut inventory.0, index) {
        Some(kind) => last.show(format!(
            "CRAFTED {} - {}",
            kind.label().to_uppercase(),
            kind.purpose()
        )),
        None => last.show("Could not craft that item"),
    }
}
