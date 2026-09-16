//! Bevy wiring for inventory, pickups, selected-item use and crafting.
//! Feedback is intentionally short: the HUD should teach without narrating.

use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::Body;
use crate::player::Player;
use crate::survival::Survival;

use super::craft;
use super::inventory::Inventory;
use super::item::{ItemKind, ItemStack};

const DEFAULT_CAPACITY: u32 = 12;
const PICKUP_RADIUS: f32 = 40.0;
const EVENT_DISPLAY_SECONDS: f32 = 2.6;
const EVENT_TIMER_RATE: f32 = 0.62;

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
        last.remaining = (last.remaining - time.delta_secs() * EVENT_TIMER_RATE).max(0.0);
    }
}

fn collect_pickups(
    virtual_time: Res<Time<Virtual>>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    pickups: Query<(Entity, &Transform, &Pickup)>,
    mut inventory: Query<&mut PlayerInventory>,
    mut selected: ResMut<SelectedSlot>,
    mut last: ResMut<LastEvent>,
) {
    if virtual_time.is_paused() {
        return;
    }
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
        inventory.0.add(pickup.0);
        if let Some(index) = inventory
            .0
            .stacks()
            .iter()
            .position(|stack| stack.kind == pickup.0.kind)
        {
            selected.0 = index.min(8);
        }
        last.show(format!(
            "PICKUP  {} x{}",
            pickup.0.kind.label().to_uppercase(),
            pickup.0.quantity
        ));
        commands.entity(entity).despawn();
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
    virtual_time: Res<Time<Virtual>>,
    crafting: Res<CraftingMenu>,
    mut selected: ResMut<SelectedSlot>,
) {
    if crafting.open || virtual_time.is_paused() {
        return;
    }
    if let Some(slot) = digit_pressed(&keyboard) {
        selected.0 = slot;
    }
}

fn use_selected_item(
    keyboard: Res<ButtonInput<KeyCode>>,
    virtual_time: Res<Time<Virtual>>,
    crafting: Res<CraftingMenu>,
    selected: Res<SelectedSlot>,
    mut query: Query<(&mut PlayerInventory, &mut Body, &mut Survival)>,
    mut last: ResMut<LastEvent>,
) {
    if crafting.open || virtual_time.is_paused() || !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok((mut inventory, mut body, mut survival)) = query.single_mut() else {
        return;
    };
    let Some(stack) = inventory.0.stacks().get(selected.0).copied() else {
        last.show("EMPTY SLOT");
        return;
    };

    match stack.kind {
        ItemKind::Food => {
            inventory.0.remove(ItemKind::Food, 1);
            survival.0.eat(0.4);
            last.show("FOOD USED  HUNGER +40%");
        }
        ItemKind::Water => {
            inventory.0.remove(ItemKind::Water, 1);
            survival.0.drink(0.4);
            last.show("WATER USED  THIRST +40%");
        }
        ItemKind::Bandage => {
            let bleeding = body.0.total_bleed_rate() > 0.0005;
            let injured = body.0.blood_volume() < 0.999;
            if !bleeding && !injured {
                last.show("BANDAGE NOT NEEDED");
            } else {
                inventory.0.remove(ItemKind::Bandage, 1);
                body.0.treat_all_bleeding();
                body.0.heal_blood_volume(0.12);
                last.show("BANDAGED  BLEEDING STOPPED  HEALTH +12%");
            }
        }
        ItemKind::Splint => {
            if !body.0.has_untreated_leg_fracture() {
                last.show("SPLINT NOT NEEDED");
            } else {
                inventory.0.remove(ItemKind::Splint, 1);
                body.0.treat_fracture();
                last.show("SPLINT APPLIED");
            }
        }
        ItemKind::Medkit => {
            if body.0.blood_volume() >= 0.99 && body.0.wound_count() == 0 {
                last.show("MEDKIT NOT NEEDED");
            } else {
                inventory.0.remove(ItemKind::Medkit, 1);
                body.0.heal_blood_volume(0.35);
                body.0.treat_pain();
                last.show("MEDKIT USED  HEALTH +35%");
            }
        }
        ItemKind::Scrap | ItemKind::Cloth | ItemKind::Metal | ItemKind::Battery => {
            last.show("CRAFTING MATERIAL  PRESS C");
        }
    }
}

fn crafting_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    virtual_time: Res<Time<Virtual>>,
    mut crafting: ResMut<CraftingMenu>,
    mut inventory: Query<&mut PlayerInventory>,
    mut last: ResMut<LastEvent>,
) {
    // When some other modal has paused the game, do not let C silently open a
    // crafting screen behind it. If crafting itself is the reason time is
    // paused, controls remain active so the player can craft or close it.
    if virtual_time.is_paused() && !crafting.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        crafting.open = false;
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        crafting.open = !crafting.open;
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
        last.show("MISSING MATERIALS");
        return;
    }
    match craft::try_craft_index(&mut inventory.0, index) {
        Some(kind) => last.show(format!("CRAFTED  {}", kind.label().to_uppercase())),
        None => last.show("CRAFT FAILED"),
    }
}
