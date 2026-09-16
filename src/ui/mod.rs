//! Player-facing HUD. The UI deliberately explains simulation state instead
//! of forcing the player to infer it: large labelled vitals, explicit damage
//! causes, a persistent objective, fixed inventory slots, recipe purposes,
//! contextual event toasts, and clean death/win transitions.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::items::{recipe_descriptions, LastEvent, PlayerInventory};
use crate::physics::{Grounded, LandingImpact};
use crate::player::{Player, SpawnHint};
use crate::survival::Survival;
use crate::world::{CurrentDepth, DepthAnnouncement, RunStats};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLanding>()
            .init_resource::<DebugVisible>()
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    reset_always_visible,
                    toggle_debug,
                    capture_landings,
                    update_debug_text,
                    update_vital_bars,
                    update_vital_values,
                    update_survival_warning,
                    update_inventory_panel,
                    update_inventory_header,
                    update_depth_label,
                    update_spawn_hint_toast,
                    update_depth_toast,
                    update_event_toast,
                    update_death_overlay,
                    update_win_overlay,
                    enforce_death_hide,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
struct HideOnDeath;

/// Gameplay HUD elements that should reappear automatically after R reset.
#[derive(Component)]
struct AlwaysVisible;

#[derive(Resource, Default)]
struct LastLanding {
    text: String,
}

#[derive(Resource, Default)]
struct DebugVisible(bool);

#[derive(Component)]
struct DebugPanel;

#[derive(Component, Clone, Copy, PartialEq)]
enum VitalKind {
    Health,
    Hunger,
    Thirst,
    Stamina,
}

#[derive(Component)]
struct VitalFill(VitalKind);

#[derive(Component)]
struct VitalValue(VitalKind);

#[derive(Component)]
struct SurvivalWarning;

#[derive(Component)]
struct DepthLabel;

#[derive(Component)]
struct InventoryHeader;

#[derive(Component)]
struct InventoryIcon(usize);

#[derive(Component)]
struct InventoryQty(usize);

#[derive(Component)]
struct DeathOverlay;
#[derive(Component)]
struct DeathOverlayText;

#[derive(Component)]
struct WinOverlay;
#[derive(Component)]
struct WinOverlayText;

#[derive(Component)]
struct SpawnHintPanel;

#[derive(Component)]
struct DepthToastPanel;
#[derive(Component)]
struct DepthToastText;

#[derive(Component)]
struct EventToastPanel;
#[derive(Component)]
struct EventToastText;

fn panel_bg() -> Color {
    Color::srgba(0.06, 0.055, 0.05, 0.84)
}

fn bar_bg() -> Color {
    Color::srgb(0.10, 0.09, 0.09)
}

fn slot_bg() -> Color {
    Color::srgba(0.16, 0.15, 0.14, 0.94)
}

const INVENTORY_SLOTS: usize = 9;
const INVENTORY_COLS: usize = 3;
const INVENTORY_SLOT: f32 = 58.0;
const INVENTORY_GAP: f32 = 8.0;
const VITAL_BAR_CONTENT_WIDTH: f32 = 282.0;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_debug_panel(&mut commands);
    spawn_vitals_panel(&mut commands, &asset_server);
    spawn_inventory_panel(&mut commands, &asset_server);
    spawn_recipe_panel(&mut commands);
    spawn_objective_and_depth(&mut commands);
    spawn_toasts(&mut commands);
    spawn_death_overlay(&mut commands);
    spawn_win_overlay(&mut commands);
}

fn spawn_debug_panel(commands: &mut Commands) {
    commands.spawn((
        DebugPanel,
        HideOnDeath,
        Text::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgb(0.65, 0.85, 0.65)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Px(18.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
    ));
}

fn spawn_vitals_panel(commands: &mut Commands, asset_server: &AssetServer) {
    const PANEL_LEFT: f32 = 18.0;
    const PANEL_BOTTOM: f32 = 18.0;
    const ICON_LEFT: f32 = 28.0;
    const LABEL_LEFT: f32 = 74.0;
    const BAR_LEFT: f32 = 154.0;
    const OUTLINE_WIDTH: f32 = 292.0;
    const BORDER: f32 = 5.0;
    const ROW_HEIGHT: f32 = 38.0;
    const ROW_GAP: f32 = 8.0;
    const FIRST_BOTTOM: f32 = 24.0;
    const PANEL_WIDTH: f32 = 462.0;
    const PANEL_HEIGHT: f32 = 208.0;

    commands.spawn((
        HideOnDeath,
        AlwaysVisible,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(PANEL_LEFT),
            bottom: Val::Px(PANEL_BOTTOM),
            width: Val::Px(PANEL_WIDTH),
            height: Val::Px(PANEL_HEIGHT),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    let rows = [
        (VitalKind::Health, "HEALTH", "sprites/icon_health.png", Color::srgb(0.30, 0.82, 0.34)),
        (VitalKind::Hunger, "HUNGER", "sprites/icon_hunger.png", Color::srgb(0.85, 0.62, 0.22)),
        (VitalKind::Thirst, "THIRST", "sprites/icon_thirst.png", Color::srgb(0.30, 0.62, 0.92)),
        (VitalKind::Stamina, "STAMINA", "sprites/icon_stamina.png", Color::srgb(0.68, 0.85, 0.32)),
    ];

    for (index, (kind, label, icon_path, color)) in rows.into_iter().enumerate() {
        let bottom = FIRST_BOTTOM + index as f32 * (ROW_HEIGHT + ROW_GAP);

        commands.spawn((
            HideOnDeath,
            AlwaysVisible,
            ImageNode::new(asset_server.load(icon_path)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ICON_LEFT),
                bottom: Val::Px(bottom + 2.0),
                width: Val::Px(34.0),
                height: Val::Px(34.0),
                ..default()
            },
        ));

        commands.spawn((
            HideOnDeath,
            AlwaysVisible,
            Text::new(label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.88, 0.86, 0.79)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LABEL_LEFT),
                bottom: Val::Px(bottom + 8.0),
                ..default()
            },
        ));

        commands.spawn((
            HideOnDeath,
            AlwaysVisible,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT),
                bottom: Val::Px(bottom),
                width: Val::Px(OUTLINE_WIDTH),
                height: Val::Px(ROW_HEIGHT),
                ..default()
            },
            BackgroundColor(Color::srgb(0.42, 0.39, 0.34)),
        ));

        commands.spawn((
            HideOnDeath,
            AlwaysVisible,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT + BORDER),
                bottom: Val::Px(bottom + BORDER),
                width: Val::Px(VITAL_BAR_CONTENT_WIDTH),
                height: Val::Px(ROW_HEIGHT - 2.0 * BORDER),
                ..default()
            },
            BackgroundColor(bar_bg()),
        ));

        commands.spawn((
            VitalFill(kind),
            HideOnDeath,
            AlwaysVisible,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT + BORDER),
                bottom: Val::Px(bottom + BORDER),
                width: Val::Px(VITAL_BAR_CONTENT_WIDTH),
                height: Val::Px(ROW_HEIGHT - 2.0 * BORDER),
                ..default()
            },
            BackgroundColor(color),
        ));

        commands.spawn((
            VitalValue(kind),
            HideOnDeath,
            AlwaysVisible,
            Text::new("100%"),
            TextFont { font_size: 15.0, ..default() },
            TextColor(Color::srgb(0.97, 0.96, 0.92)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT + 118.0),
                bottom: Val::Px(bottom + 9.0),
                ..default()
            },
        ));
    }

    commands.spawn((
        SurvivalWarning,
        HideOnDeath,
        Text::new(""),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.96, 0.58, 0.32)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(PANEL_LEFT + 4.0),
            bottom: Val::Px(PANEL_BOTTOM + PANEL_HEIGHT + 8.0),
            padding: UiRect::all(Val::Px(7.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.03, 0.02, 0.86)),
    ));
}

fn inventory_panel_dimensions() -> (f32, f32) {
    let rows = (INVENTORY_SLOTS + INVENTORY_COLS - 1) / INVENTORY_COLS;
    let width = INVENTORY_COLS as f32 * INVENTORY_SLOT
        + (INVENTORY_COLS as f32 - 1.0) * INVENTORY_GAP
        + 20.0;
    let height = rows as f32 * INVENTORY_SLOT
        + (rows as f32 - 1.0) * INVENTORY_GAP
        + 52.0;
    (width, height)
}

fn spawn_inventory_panel(commands: &mut Commands, asset_server: &AssetServer) {
    let (panel_w, panel_h) = inventory_panel_dimensions();

    commands.spawn((
        HideOnDeath,
        AlwaysVisible,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            bottom: Val::Px(18.0),
            width: Val::Px(panel_w),
            height: Val::Px(panel_h),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    commands.spawn((
        InventoryHeader,
        HideOnDeath,
        AlwaysVisible,
        Text::new("INVENTORY"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.90, 0.87, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(28.0),
            bottom: Val::Px(18.0 + panel_h - 33.0),
            ..default()
        },
    ));

    for i in 0..INVENTORY_SLOTS {
        let col = i % INVENTORY_COLS;
        let row = i / INVENTORY_COLS;
        let right = 18.0 + 10.0 + col as f32 * (INVENTORY_SLOT + INVENTORY_GAP);
        let bottom = 18.0 + 10.0 + row as f32 * (INVENTORY_SLOT + INVENTORY_GAP);

        commands.spawn((
            HideOnDeath,
            AlwaysVisible,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(right),
                bottom: Val::Px(bottom),
                width: Val::Px(INVENTORY_SLOT),
                height: Val::Px(INVENTORY_SLOT),
                ..default()
            },
            BackgroundColor(slot_bg()),
        ));

        commands.spawn((
            InventoryIcon(i),
            HideOnDeath,
            ImageNode::new(asset_server.load("sprites/item_scrap.png")),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                right: Val::Px(right + 6.0),
                bottom: Val::Px(bottom + 6.0),
                width: Val::Px(INVENTORY_SLOT - 12.0),
                height: Val::Px(INVENTORY_SLOT - 12.0),
                ..default()
            },
        ));

        commands.spawn((
            InventoryQty(i),
            HideOnDeath,
            Text::new(""),
            TextFont { font_size: 17.0, ..default() },
            TextColor(Color::srgb(0.98, 0.96, 0.90)),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                right: Val::Px(right + 3.0),
                bottom: Val::Px(bottom + 2.0),
                ..default()
            },
        ));
    }
}

fn spawn_recipe_panel(commands: &mut Commands) {
    let (_, inventory_h) = inventory_panel_dimensions();
    let recipes = recipe_descriptions();
    let height = 52.0 + recipes.len() as f32 * 42.0;

    commands
        .spawn((
            HideOnDeath,
            AlwaysVisible,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                bottom: Val::Px(18.0 + inventory_h + 12.0),
                width: Val::Px(430.0),
                height: Val::Px(height),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(panel_bg()),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("CRAFTING  •  C crafts the first affordable recipe"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.91, 0.84, 0.63)),
            ));
            for recipe in recipes {
                panel.spawn((
                    Text::new(recipe),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::srgb(0.80, 0.77, 0.69)),
                ));
            }
        });
}

fn spawn_objective_and_depth(commands: &mut Commands) {
    commands.spawn((
        HideOnDeath,
        AlwaysVisible,
        Text::new("OBJECTIVE  •  descend to the bottom and recover the cargo"),
        TextFont { font_size: 19.0, ..default() },
        TextColor(Color::srgb(0.96, 0.88, 0.60)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Px(20.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.045, 0.03, 0.78)),
    ));

    commands.spawn((
        DepthLabel,
        HideOnDeath,
        AlwaysVisible,
        Text::new("LAYER 0"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgba(0.88, 0.83, 0.72, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(24.0),
            ..default()
        },
    ));
}

fn spawn_toasts(commands: &mut Commands) {
    commands
        .spawn((
            SpawnHintPanel,
            HideOnDeath,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(66.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Explore downward, collect supplies, treat injuries, recover the cargo."),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgba(0.95, 0.92, 0.82, 0.98)),
            ));
            panel.spawn((
                Text::new("A/D or arrows: move   Space: jump   E: attack   F: use best supply   C: craft   R: restart"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.88, 0.86, 0.80, 0.95)),
            ));
        });

    commands
        .spawn((
            DepthToastPanel,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(118.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                DepthToastText,
                Text::new(""),
                TextFont { font_size: 25.0, ..default() },
                TextColor(Color::srgba(0.88, 0.84, 0.72, 0.98)),
            ));
        });

    commands
        .spawn((
            EventToastPanel,
            HideOnDeath,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                bottom: Val::Px(238.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                EventToastText,
                Text::new(""),
                TextFont { font_size: 19.0, ..default() },
                TextColor(Color::srgba(0.95, 0.92, 0.84, 0.98)),
            ));
        });
}

fn spawn_death_overlay(commands: &mut Commands) {
    commands
        .spawn((
            DeathOverlay,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.01, 0.01, 0.97)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("YOU DIED"),
                TextFont { font_size: 62.0, ..default() },
                TextColor(Color::srgb(0.78, 0.20, 0.18)),
            ));
            overlay.spawn((
                DeathOverlayText,
                Text::new(""),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.84, 0.81, 0.75)),
            ));
        });
}

fn spawn_win_overlay(commands: &mut Commands) {
    commands
        .spawn((
            WinOverlay,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.01, 0.97)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("CARGO RECOVERED"),
                TextFont { font_size: 54.0, ..default() },
                TextColor(Color::srgb(0.50, 0.78, 0.37)),
            ));
            overlay.spawn((
                WinOverlayText,
                Text::new(""),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.84, 0.81, 0.75)),
            ));
        });
}

fn reset_always_visible(mut query: Query<&mut Node, With<AlwaysVisible>>) {
    for mut node in &mut query {
        node.display = Display::Flex;
    }
}

fn toggle_debug(keyboard: Res<ButtonInput<KeyCode>>, mut visible: ResMut<DebugVisible>) {
    if keyboard.just_pressed(KeyCode::F3) {
        visible.0 = !visible.0;
    }
}

fn capture_landings(mut events: MessageReader<LandingImpact>, mut last: ResMut<LastLanding>) {
    for impact in events.read() {
        last.text = format!(
            "hard landing  {:.0} u/s  severity {:.2}",
            impact.downward_speed, impact.severity
        );
    }
}

fn update_debug_text(
    visible: Res<DebugVisible>,
    player: Query<(&LinearVelocity, Option<&Grounded>, &Body, &PlayerInventory), With<Player>>,
    last: Res<LastLanding>,
    depth: Res<CurrentDepth>,
    mut panel: Query<(&mut Node, &mut Text), With<DebugPanel>>,
) {
    let Ok((mut node, mut text)) = panel.single_mut() else {
        return;
    };
    if !visible.0 {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;
    let Ok((velocity, grounded, body, inventory)) = player.single() else {
        return;
    };
    let grounded = if grounded.is_some() { "grounded" } else { "airborne" };
    let items = if inventory.0.stacks().is_empty() {
        "empty".to_string()
    } else {
        inventory
            .0
            .stacks()
            .iter()
            .map(|s| format!("{} {}", s.quantity, s.kind.label()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    **text = format!(
        "[debug — F3 to hide]\n\
         depth layer {}   {grounded}   vel ({:.0}, {:.0})\n\
         wounds {}   pain {:.1}   bleed {:.2}/s   blood {:.0}%\n\
         inventory ({}/{}): {items}\n\
         {}",
        depth.0,
        velocity.x,
        velocity.y,
        body.0.wound_count(),
        body.0.total_pain(),
        body.0.total_bleed_rate(),
        body.0.blood_volume() * 100.0,
        inventory.0.used_weight(),
        inventory.0.capacity(),
        last.text,
    );
}

fn vital_fraction(kind: VitalKind, body: &Body, survival: &Survival) -> f32 {
    match kind {
        VitalKind::Health => body.0.blood_volume(),
        VitalKind::Hunger => survival.0.hunger(),
        VitalKind::Thirst => survival.0.thirst(),
        VitalKind::Stamina => survival.0.stamina(),
    }
}

fn update_vital_bars(
    player: Query<(&Body, &Survival), With<Player>>,
    mut fills: Query<(&VitalFill, &mut Node)>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    for (fill, mut node) in &mut fills {
        let fraction = vital_fraction(fill.0, body, survival);
        node.width = Val::Px(VITAL_BAR_CONTENT_WIDTH * fraction.clamp(0.0, 1.0));
    }
}

fn update_vital_values(
    player: Query<(&Body, &Survival), With<Player>>,
    mut values: Query<(&VitalValue, &mut Text)>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    for (value, mut text) in &mut values {
        **text = format!("{:.0}%", vital_fraction(value.0, body, survival) * 100.0);
    }
}

/// Keep the warning line causal. Health only falls because of bleeding now;
/// hunger/thirst warn about exhaustion rather than pretending to be damage.
fn update_survival_warning(
    player: Query<(&Body, &Survival), With<Player>>,
    mut warning: Query<(&mut Node, &mut Text), With<SurvivalWarning>>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    let Ok((mut node, mut text)) = warning.single_mut() else {
        return;
    };

    let message = if body.0.total_bleed_rate() > 0.0 {
        Some(format!(
            "BLEEDING — health is falling ({:.2}/s). Use a bandage with F.",
            body.0.total_bleed_rate()
        ))
    } else if body.0.has_untreated_leg_fracture() {
        Some("FRACTURE — movement is limited. Carry a splint and press F.".to_string())
    } else if survival.0.is_starving() && survival.0.is_dehydrated() {
        Some("STARVING & DEHYDRATED — stamina recovery is heavily reduced.".to_string())
    } else if survival.0.is_starving() {
        Some("STARVING — stamina recovery is reduced. Find food.".to_string())
    } else if survival.0.is_dehydrated() {
        Some("DEHYDRATED — stamina recovery is reduced. Find water.".to_string())
    } else if survival.0.hunger() < 0.3 || survival.0.thirst() < 0.3 {
        Some("LOW SUPPLIES — hunger/thirst are starting to hurt stamina recovery.".to_string())
    } else {
        None
    };

    match message {
        Some(value) => {
            node.display = Display::Flex;
            **text = value;
        }
        None => node.display = Display::None,
    }
}

fn update_inventory_panel(
    player: Query<&PlayerInventory, With<Player>>,
    asset_server: Res<AssetServer>,
    mut icons: Query<(&InventoryIcon, &mut Node, &mut ImageNode), Without<InventoryQty>>,
    mut qtys: Query<(&InventoryQty, &mut Node, &mut Text), Without<InventoryIcon>>,
) {
    let Ok(inventory) = player.single() else {
        return;
    };
    let stacks = inventory.0.stacks();
    for (slot, mut node, mut image) in &mut icons {
        if let Some(stack) = stacks.get(slot.0) {
            node.display = Display::Flex;
            image.image = asset_server.load(stack.kind.sprite_path());
        } else {
            node.display = Display::None;
        }
    }
    for (slot, mut node, mut text) in &mut qtys {
        if let Some(stack) = stacks.get(slot.0) {
            node.display = Display::Flex;
            **text = format!("{}", stack.quantity);
        } else {
            node.display = Display::None;
        }
    }
}

fn update_inventory_header(
    player: Query<&PlayerInventory, With<Player>>,
    mut header: Query<&mut Text, With<InventoryHeader>>,
) {
    let Ok(inventory) = player.single() else {
        return;
    };
    let Ok(mut text) = header.single_mut() else {
        return;
    };
    **text = format!(
        "INVENTORY  •  {}/{} weight",
        inventory.0.used_weight(),
        inventory.0.capacity()
    );
}

fn update_depth_label(depth: Res<CurrentDepth>, mut label: Query<&mut Text, With<DepthLabel>>) {
    let Ok(mut text) = label.single_mut() else {
        return;
    };
    **text = format!("LAYER {}", depth.0);
}

fn update_spawn_hint_toast(hint: Res<SpawnHint>, mut panel: Query<&mut Node, With<SpawnHintPanel>>) {
    let Ok(mut node) = panel.single_mut() else {
        return;
    };
    node.display = if hint.0 > 0.0 {
        Display::Flex
    } else {
        Display::None
    };
}

fn update_depth_toast(
    announcement: Res<DepthAnnouncement>,
    mut panel: Query<&mut Node, With<DepthToastPanel>>,
    mut text: Query<&mut Text, With<DepthToastText>>,
) {
    let Ok(mut node) = panel.single_mut() else {
        return;
    };
    if announcement.remaining > 0.0 {
        node.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            **text = announcement.text.clone();
        }
    } else {
        node.display = Display::None;
    }
}

fn update_event_toast(
    last: Res<LastEvent>,
    mut panel: Query<&mut Node, With<EventToastPanel>>,
    mut text: Query<&mut Text, With<EventToastText>>,
) {
    let Ok(mut node) = panel.single_mut() else {
        return;
    };
    if last.remaining > 0.0 {
        node.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            **text = last.text.clone();
        }
    } else {
        node.display = Display::None;
    }
}

fn update_death_overlay(
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    mut overlay: Query<&mut Node, With<DeathOverlay>>,
    mut overlay_text: Query<&mut Text, With<DeathOverlayText>>,
) {
    let Ok(body) = player.single() else {
        return;
    };
    let Ok(mut overlay_node) = overlay.single_mut() else {
        return;
    };
    if body.0.is_dead() && !stats.extracted {
        overlay_node.display = Display::Flex;
        if let Ok(mut text) = overlay_text.single_mut() {
            **text = format!(
                "cause: critical blood loss from untreated injuries\n\
                 wounds at death: {}\n\
                 deepest layer reached: {}\n\
                 survived: {:.0}s\n\n\
                 press R to restart with a new cave",
                body.0.wound_count(),
                stats.deepest_layer,
                stats.elapsed_secs
            );
        }
    } else {
        overlay_node.display = Display::None;
    }
}

fn update_win_overlay(
    stats: Res<RunStats>,
    mut overlay: Query<&mut Node, With<WinOverlay>>,
    mut overlay_text: Query<&mut Text, With<WinOverlayText>>,
) {
    let Ok(mut overlay_node) = overlay.single_mut() else {
        return;
    };
    if stats.extracted {
        overlay_node.display = Display::Flex;
        if let Ok(mut text) = overlay_text.single_mut() {
            **text = format!(
                "primary objective complete\n\
                 deepest layer reached: {}\n\
                 survived: {:.0}s\n\n\
                 press R to begin a new run",
                stats.deepest_layer, stats.elapsed_secs
            );
        }
    } else {
        overlay_node.display = Display::None;
    }
}

/// Runs last so overlays cannot have normal HUD elements drawn through them.
/// reset_always_visible runs first next frame, which guarantees that R reset
/// restores the HUD instead of leaving it permanently hidden.
fn enforce_death_hide(
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    mut hidden: Query<&mut Node, With<HideOnDeath>>,
) {
    let dead = player.single().map(|b| b.0.is_dead()).unwrap_or(false);
    if !(dead || stats.extracted) {
        return;
    }
    for mut node in &mut hidden {
        node.display = Display::None;
    }
}
