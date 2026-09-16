//! HUD: a bottom-left vitals cluster (icon + bar per stat, plus a
//! warning line explaining any passive health drain), a bottom-right
//! fixed-grid inventory (always shows all slots, Minecraft-style) with
//! a recipe reference panel above it, a persistent depth label, fading
//! toasts for spawn instructions/objective/depth changes/events, a
//! developer debug overlay (F3, hidden by default), and full-screen
//! death/win overlays that hide everything else while active.

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
                    update_stamina_visibility,
                    update_survival_warning,
                    update_inventory_panel,
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

/// entities tagged with this are forced to Display::None while dead or
/// extracted, regardless of whatever their own update system set this
/// frame — enforce_death_hide runs last in the chain specifically to
/// win that fight.
#[derive(Component)]
struct HideOnDeath;

/// entities tagged with this have NO per-frame system that otherwise
/// sets their Display — reset_always_visible force-shows them at the
/// START of every frame, so enforce_death_hide's None (while dead) gets
/// correctly undone again on the next frame once alive. Without this,
/// anything only ever hidden once by enforce_death_hide would stay
/// hidden forever after the first death — a real bug an earlier version
/// of this file had for the vitals panel, the depth label, and more.
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
struct VitalPart(VitalKind);

#[derive(Component)]
struct SurvivalWarning;

#[derive(Component)]
struct DepthLabel;

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
    Color::srgba(0.06, 0.055, 0.05, 0.68)
}

fn bar_bg() -> Color {
    Color::srgb(0.10, 0.09, 0.09)
}

fn slot_bg() -> Color {
    Color::srgba(0.16, 0.15, 0.14, 0.9)
}

const INVENTORY_SLOTS: usize = 9;
const INVENTORY_COLS: usize = 3;
const INVENTORY_SLOT: f32 = 46.0;
const INVENTORY_GAP: f32 = 6.0;
/// bar outline width minus 2x border — see spawn_vitals_panel. kept in
/// sync with it manually; if you resize the bars there, update this too.
const VITAL_BAR_CONTENT_WIDTH: f32 = 222.0;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_debug_panel(&mut commands);
    spawn_vitals_panel(&mut commands, &asset_server);
    spawn_inventory_panel(&mut commands, &asset_server);
    spawn_recipe_panel(&mut commands);
    spawn_depth_label(&mut commands);
    spawn_toasts(&mut commands);
    spawn_death_overlay(&mut commands);
    spawn_win_overlay(&mut commands);
}

fn spawn_debug_panel(commands: &mut Commands) {
    commands.spawn((
        DebugPanel,
        HideOnDeath,
        Text::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(0.65, 0.85, 0.65)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
    ));
}

fn spawn_vitals_panel(commands: &mut Commands, asset_server: &AssetServer) {
    const PANEL_LEFT: f32 = 14.0;
    const PANEL_BOTTOM: f32 = 14.0;
    const ICON_LEFT: f32 = 20.0;
    const BAR_LEFT: f32 = 64.0;
    const OUTLINE_WIDTH: f32 = 230.0;
    const BORDER: f32 = 4.0;
    const ROW_GAP: f32 = 8.0;
    const HEALTH_BOTTOM: f32 = 26.0;
    const HEALTH_HEIGHT: f32 = 34.0;
    const HEALTH_ICON: f32 = 30.0;
    const MINOR_HEIGHT: f32 = 18.0;
    const MINOR_ICON: f32 = 22.0;
    const PANEL_WIDTH: f32 = BAR_LEFT + OUTLINE_WIDTH + 16.0;
    const PANEL_HEIGHT: f32 = 150.0;

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

    let hunger_bottom = HEALTH_BOTTOM + HEALTH_HEIGHT + ROW_GAP;
    let thirst_bottom = hunger_bottom + MINOR_HEIGHT + ROW_GAP;
    let stamina_bottom = thirst_bottom + MINOR_HEIGHT + ROW_GAP;

    // (kind, icon, bar color, bottom offset, outline height, icon size,
    // always-visible-when-alive). health is the biggest/brightest — the
    // stat that matters most, kept green per the original design ask.
    let rows: [(VitalKind, &str, Color, f32, f32, f32, bool); 4] = [
        (VitalKind::Health, "sprites/icon_health.png", Color::srgb(0.30, 0.82, 0.34), HEALTH_BOTTOM, HEALTH_HEIGHT, HEALTH_ICON, true),
        (VitalKind::Hunger, "sprites/icon_hunger.png", Color::srgb(0.85, 0.62, 0.22), hunger_bottom, MINOR_HEIGHT, MINOR_ICON, true),
        (VitalKind::Thirst, "sprites/icon_thirst.png", Color::srgb(0.30, 0.62, 0.92), thirst_bottom, MINOR_HEIGHT, MINOR_ICON, true),
        (VitalKind::Stamina, "sprites/icon_stamina.png", Color::srgb(0.68, 0.85, 0.32), stamina_bottom, MINOR_HEIGHT, MINOR_ICON, false),
    ];
    let outline_color = Color::srgb(0.42, 0.39, 0.34);
    let content_width = OUTLINE_WIDTH - 2.0 * BORDER;

    for (kind, icon_path, color, bottom, outline_height, icon_size, always_visible) in rows {
        // icon, outline, background all share the same Display fate as
        // each other — only "always_visible" (health/hunger/thirst) get
        // the AlwaysVisible tag; stamina keeps its own dedicated
        // show-while-moving system (update_stamina_visibility) instead.
        let mut icon = commands.spawn((
            VitalPart(kind),
            HideOnDeath,
            ImageNode::new(asset_server.load(icon_path)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ICON_LEFT),
                bottom: Val::Px(bottom - (icon_size - outline_height) / 2.0),
                width: Val::Px(icon_size),
                height: Val::Px(icon_size),
                ..default()
            },
        ));
        if always_visible {
            icon.insert(AlwaysVisible);
        }

        let mut outline = commands.spawn((
            VitalPart(kind),
            HideOnDeath,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT),
                bottom: Val::Px(bottom),
                width: Val::Px(OUTLINE_WIDTH),
                height: Val::Px(outline_height),
                ..default()
            },
            BackgroundColor(outline_color),
        ));
        if always_visible {
            outline.insert(AlwaysVisible);
        }

        let mut bg = commands.spawn((
            VitalPart(kind),
            HideOnDeath,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT + BORDER),
                bottom: Val::Px(bottom + BORDER),
                width: Val::Px(content_width),
                height: Val::Px(outline_height - 2.0 * BORDER),
                ..default()
            },
            BackgroundColor(bar_bg()),
        ));
        if always_visible {
            bg.insert(AlwaysVisible);
        }

        let mut fill = commands.spawn((
            VitalFill(kind),
            HideOnDeath,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BAR_LEFT + BORDER),
                bottom: Val::Px(bottom + BORDER),
                width: Val::Px(content_width),
                height: Val::Px(outline_height - 2.0 * BORDER),
                ..default()
            },
            BackgroundColor(color),
        ));
        if always_visible {
            fill.insert(AlwaysVisible);
        }
    }

    // warning line, just above the panel — explains any passive health
    // drain from starving/dehydration instead of leaving it a mystery.
    commands.spawn((
        SurvivalWarning,
        HideOnDeath,
        Text::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgb(0.88, 0.55, 0.30)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(PANEL_LEFT + 2.0),
            bottom: Val::Px(PANEL_BOTTOM + PANEL_HEIGHT + 6.0),
            ..default()
        },
    ));
}

fn spawn_inventory_panel(commands: &mut Commands, asset_server: &AssetServer) {
    const ROWS: usize = (INVENTORY_SLOTS + INVENTORY_COLS - 1) / INVENTORY_COLS;
    const PANEL_W: f32 = INVENTORY_COLS as f32 * INVENTORY_SLOT + (INVENTORY_COLS as f32 - 1.0) * INVENTORY_GAP + 16.0;
    const PANEL_H: f32 = ROWS as f32 * INVENTORY_SLOT + (ROWS as f32 - 1.0) * INVENTORY_GAP + 16.0;

    commands.spawn((
        HideOnDeath,
        AlwaysVisible,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            bottom: Val::Px(14.0),
            width: Val::Px(PANEL_W),
            height: Val::Px(PANEL_H),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    for i in 0..INVENTORY_SLOTS {
        let col = i % INVENTORY_COLS;
        let row = i / INVENTORY_COLS;
        let right = 14.0 + 8.0 + col as f32 * (INVENTORY_SLOT + INVENTORY_GAP);
        let bottom = 14.0 + 8.0 + row as f32 * (INVENTORY_SLOT + INVENTORY_GAP);

        // slot background — always shown, empty or not, Minecraft-style,
        // so the inventory reads as a fixed grid instead of things
        // popping in and out of nowhere.
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
                right: Val::Px(right + 5.0),
                bottom: Val::Px(bottom + 5.0),
                width: Val::Px(INVENTORY_SLOT - 10.0),
                height: Val::Px(INVENTORY_SLOT - 10.0),
                ..default()
            },
        ));

        commands.spawn((
            InventoryQty(i),
            HideOnDeath,
            Text::new(""),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.95, 0.93, 0.88)),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                right: Val::Px(right + 2.0),
                bottom: Val::Px(bottom + 2.0),
                ..default()
            },
        ));
    }
}

/// static reference panel above the inventory — "what crafts what and
/// why" was invisible before; this lists every known recipe so there's
/// a reason to go collect scrap/cloth/metal/battery.
fn spawn_recipe_panel(commands: &mut Commands) {
    const ROWS: usize = (INVENTORY_SLOTS + INVENTORY_COLS - 1) / INVENTORY_COLS;
    const INVENTORY_PANEL_H: f32 =
        ROWS as f32 * INVENTORY_SLOT + (ROWS as f32 - 1.0) * INVENTORY_GAP + 16.0;
    let bottom = 14.0 + INVENTORY_PANEL_H + 10.0;
    let recipes = recipe_descriptions();
    let height = 34.0 + recipes.len() as f32 * 18.0;

    commands.spawn((
        HideOnDeath,
        AlwaysVisible,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            bottom: Val::Px(bottom),
            width: Val::Px(280.0),
            height: Val::Px(height),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new("CRAFTING (press C)"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.85, 0.82, 0.72)),
        ));
        for recipe in recipes {
            panel.spawn((
                Text::new(recipe),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.72, 0.69, 0.62)),
            ));
        }
    });
}

fn spawn_depth_label(commands: &mut Commands) {
    commands.spawn((
        DepthLabel,
        HideOnDeath,
        AlwaysVisible,
        Text::new("LAYER 0"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgba(0.82, 0.78, 0.70, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(20.0),
            ..default()
        },
    ));
}

fn spawn_toasts(commands: &mut Commands) {
    commands
        .spawn((
            SpawnHintPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("OBJECTIVE: descend, survive, and recover the cargo at the bottom of the cave"),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgba(0.95, 0.88, 0.60, 0.95)),
            ));
            panel.spawn((
                Text::new("WASD/arrows move   Space jump   E attack   F use supplies   C craft   R restart run"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgba(0.92, 0.90, 0.82, 0.95)),
            ));
        });

    commands
        .spawn((
            DepthToastPanel,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(74.0),
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
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgba(0.85, 0.82, 0.70, 0.95)),
            ));
        });

    commands
        .spawn((
            EventToastPanel,
            HideOnDeath,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                bottom: Val::Px(180.0),
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
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.88, 0.85, 0.78, 0.95)),
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
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.01, 0.01, 0.96)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("YOU DIED"),
                TextFont { font_size: 52.0, ..default() },
                TextColor(Color::srgb(0.72, 0.18, 0.18)),
            ));
            overlay.spawn((
                DeathOverlayText,
                Text::new(""),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(0.80, 0.77, 0.72)),
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
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.01, 0.96)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("CARGO RECOVERED"),
                TextFont { font_size: 46.0, ..default() },
                TextColor(Color::srgb(0.50, 0.74, 0.36)),
            ));
            overlay.spawn((
                WinOverlayText,
                Text::new(""),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(0.80, 0.77, 0.72)),
            ));
        });
}

/// runs first: force-shows everything tagged AlwaysVisible every frame.
/// see the AlwaysVisible doc comment for why this exists.
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
        last.text = format!("hard landing  {:.0} u/s  severity {:.2}", impact.downward_speed, impact.severity);
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
        inventory.0.stacks().iter().map(|s| format!("{} {}", s.quantity, s.kind.label())).collect::<Vec<_>>().join(", ")
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

fn update_vital_bars(player: Query<(&Body, &Survival), With<Player>>, mut fills: Query<(&VitalFill, &mut Node)>) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    for (fill, mut node) in &mut fills {
        let fraction = match fill.0 {
            VitalKind::Health => body.0.blood_volume(),
            VitalKind::Hunger => survival.0.hunger(),
            VitalKind::Thirst => survival.0.thirst(),
            VitalKind::Stamina => survival.0.stamina(),
        };
        node.width = Val::Px(VITAL_BAR_CONTENT_WIDTH * fraction.clamp(0.0, 1.0));
    }
}

/// stamina's icon/bg/fill only show up while the player is actually
/// moving — otherwise it's clutter for a meter that's almost always full.
fn update_stamina_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut parts: Query<(&VitalPart, &mut Node), Without<VitalFill>>,
    mut fills: Query<(&VitalFill, &mut Node), Without<VitalPart>>,
) {
    let moving = keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD)
        || keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::ArrowRight);
    let display = if moving { Display::Flex } else { Display::None };
    for (part, mut node) in &mut parts {
        if part.0 == VitalKind::Stamina {
            node.display = display;
        }
    }
    for (fill, mut node) in &mut fills {
        if fill.0 == VitalKind::Stamina {
            node.display = display;
        }
    }
}

/// explains WHY blood volume might be draining with no wounds in sight —
/// reported as "health decreasing for no reason". starving/dehydrated
/// (meters at exactly empty) actually drain blood; below 40% is just an
/// early warning that it's heading that way.
fn update_survival_warning(
    player: Query<&Survival, With<Player>>,
    mut warning: Query<(&mut Node, &mut Text), With<SurvivalWarning>>,
) {
    let Ok(survival) = player.single() else {
        return;
    };
    let Ok((mut node, mut text)) = warning.single_mut() else {
        return;
    };
    let starving = survival.0.is_starving();
    let dehydrated = survival.0.is_dehydrated();
    let message = if starving && dehydrated {
        Some("STARVING & DEHYDRATED — losing blood until you eat and drink".to_string())
    } else if starving {
        Some("STARVING — losing blood until you eat (F)".to_string())
    } else if dehydrated {
        Some("DEHYDRATED — losing blood until you drink (F)".to_string())
    } else if survival.0.hunger() < 0.4 && survival.0.thirst() < 0.4 {
        Some("getting hungry and thirsty".to_string())
    } else if survival.0.hunger() < 0.4 {
        Some("getting hungry".to_string())
    } else if survival.0.thirst() < 0.4 {
        Some("getting thirsty".to_string())
    } else {
        None
    };
    match message {
        Some(text_value) => {
            node.display = Display::Flex;
            **text = text_value;
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
    node.display = if hint.0 > 0.0 { Display::Flex } else { Display::None };
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
                "deepest layer reached: {}\nsurvived: {:.0}s\n\npress R to restart",
                stats.deepest_layer, stats.elapsed_secs
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
                "deepest layer reached: {}\nsurvived: {:.0}s\n\npress R to run again",
                stats.deepest_layer, stats.elapsed_secs
            );
        }
    } else {
        overlay_node.display = Display::None;
    }
}

/// runs last: while dead or extracted, forces every normal-gameplay HUD
/// element (and the debug panel, regardless of its own F3 state) off,
/// so nothing shows through the death/win overlay even at its (slightly
/// less than opaque) background alpha.
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
