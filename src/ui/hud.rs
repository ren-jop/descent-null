//! Stable retro-pixel HUD and overlay coordination.
//!
//! The normal play screen has four persistent roots only: mission, layer,
//! vitals and hotbar. Full-screen UI (pause, crafting, guide, leaderboard,
//! death/result) is coordinated so one screen cannot accidentally leave the
//! HUD hidden or let gameplay continue underneath it.

use std::fs;

use avian2d::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Virtual;

use super::{GuideState, LeaderboardState};
use crate::body::{Body, DamageCause, LastDamageCause};
use crate::items::{
    first_craftable, recipe_descriptions, CraftingMenu, LastEvent, PlayerInventory, SelectedSlot,
};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{CurrentDepth, DepthAnnouncement, RunStats};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseMenuState>()
            .init_resource::<UiFont>()
            .init_resource::<UxNoticeState>()
            .add_systems(PreStartup, load_ui_font)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    apply_ui_font,
                    assist_new_player,
                    emit_context_notices,
                    pause_controls,
                    update_objective,
                    update_layer_text,
                    update_vitals,
                    update_hotbar,
                    update_field_log,
                    update_crafting_overlay,
                    update_pause_overlay,
                    update_death_overlay,
                    enforce_gameplay_hud_visibility,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, sync_overlay_pause);
    }
}

#[derive(Resource, Default)]
struct PauseMenuState {
    open: bool,
}

#[derive(Resource, Default)]
struct UiFont(Option<Handle<Font>>);

#[derive(Resource, Default)]
struct UxNoticeState {
    moved: bool,
    movement_hint_sent: bool,
    craft_ready: Option<String>,
}

#[derive(Component)]
struct GameplayHud;
#[derive(Component)]
struct ObjectiveText;
#[derive(Component)]
struct LayerText;
#[derive(Component)]
struct FieldLogFrame;
#[derive(Component)]
struct FieldLogHeader;
#[derive(Component)]
struct FieldLogText;
#[derive(Component)]
struct FieldLogAccent;
#[derive(Component)]
struct CraftingOverlay;
#[derive(Component)]
struct CraftingText;
#[derive(Component)]
struct PauseOverlay;
#[derive(Component)]
struct DeathOverlay;
#[derive(Component)]
struct DeathText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum VitalKind {
    Health,
    Hunger,
    Thirst,
}

#[derive(Component)]
struct VitalSegment {
    kind: VitalKind,
    index: usize,
}
#[derive(Component)]
struct VitalValue(VitalKind);
#[derive(Component)]
struct HotbarSlot(usize);
#[derive(Component)]
struct HotbarIcon(usize);
#[derive(Component)]
struct HotbarQty(usize);

fn panel_bg() -> Color {
    Color::srgba(0.022, 0.020, 0.020, 0.95)
}

fn pixel_border() -> Color {
    Color::srgb(0.43, 0.39, 0.29)
}

fn pixel_highlight() -> Color {
    Color::srgb(0.80, 0.67, 0.34)
}

fn vital_color(kind: VitalKind) -> Color {
    match kind {
        VitalKind::Health => Color::srgb(0.80, 0.16, 0.16),
        VitalKind::Hunger => Color::srgb(0.78, 0.46, 0.13),
        VitalKind::Thirst => Color::srgb(0.18, 0.52, 0.90),
    }
}

fn vital_icon(kind: VitalKind) -> &'static str {
    match kind {
        VitalKind::Health => "sprites/icon_health.png",
        VitalKind::Hunger => "sprites/icon_hunger.png",
        VitalKind::Thirst => "sprites/icon_thirst.png",
    }
}

fn load_ui_font(mut fonts: ResMut<Assets<Font>>, mut ui_font: ResMut<UiFont>) {
    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/System/Applications/Utilities/Terminal.app/Contents/Resources/Fonts/SFMono-Regular.otf",
            "/Applications/Utilities/Terminal.app/Contents/Resources/Fonts/SFMono-Regular.otf",
            "/System/Library/Fonts/SFNSMono.ttf",
        ];
        for path in candidates {
            let Ok(bytes) = fs::read(path) else {
                continue;
            };
            let Ok(font) = Font::try_from_bytes(bytes) else {
                continue;
            };
            ui_font.0 = Some(fonts.add(font));
            break;
        }
    }
}

fn apply_ui_font(ui_font: Res<UiFont>, mut fonts: Query<&mut TextFont, Added<TextFont>>) {
    let Some(handle) = ui_font.0.as_ref() else {
        return;
    };
    for mut font in &mut fonts {
        font.font = handle.clone();
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_objective(&mut commands);
    spawn_layer_label(&mut commands);
    spawn_vitals(&mut commands, &asset_server);
    spawn_hotbar(&mut commands, &asset_server);
    spawn_field_log(&mut commands);
    spawn_crafting_overlay(&mut commands);
    spawn_pause_overlay(&mut commands);
    spawn_death_overlay(&mut commands);
}

fn spawn_objective(commands: &mut Commands) {
    commands
        .spawn((
            GameplayHud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                left: Val::Px(18.0),
                width: Val::Px(390.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(pixel_border()),
        ))
        .with_children(|outer| {
            outer
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg()),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        ObjectiveText,
                        Text::new("MISSION // 1 OF 2\nDESCEND + RECOVER LOST CARGO"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.87, 0.55)),
                    ));
                });
        });
}

fn spawn_layer_label(commands: &mut Commands) {
    commands
        .spawn((
            GameplayHud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                LayerText,
                Text::new("LAYER 0"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_vitals(commands: &mut Commands, asset_server: &AssetServer) {
    const PANEL_W: f32 = 302.0;
    const PANEL_H: f32 = 112.0;
    const BAR_X: f32 = 102.0;
    const SEG_W: f32 = 13.0;
    const SEG_GAP: f32 = 2.0;
    const SEG_H: f32 = 16.0;
    const ROW_H: f32 = 30.0;

    commands
        .spawn((
            GameplayHud,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                bottom: Val::Px(14.0),
                width: Val::Px(PANEL_W),
                height: Val::Px(PANEL_H),
                padding: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(pixel_border()),
        ))
        .with_children(|outer| {
            outer
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg()),
                ))
                .with_children(|panel| {
                    let rows = [
                        (VitalKind::Health, "HEALTH"),
                        (VitalKind::Hunger, "HUNGER"),
                        (VitalKind::Thirst, "THIRST"),
                    ];
                    for (row, (kind, label)) in rows.into_iter().enumerate() {
                        let y = PANEL_H - 37.0 - row as f32 * ROW_H;
                        panel.spawn((
                            ImageNode::new(asset_server.load(vital_icon(kind))),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(8.0),
                                bottom: Val::Px(y - 1.0),
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                        ));
                        panel.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.84, 0.82, 0.76)),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(35.0),
                                bottom: Val::Px(y + 1.0),
                                ..default()
                            },
                        ));
                        for index in 0..10 {
                            panel.spawn((
                                VitalSegment { kind, index },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(BAR_X - 3.0 + index as f32 * (SEG_W + SEG_GAP)),
                                    bottom: Val::Px(y),
                                    width: Val::Px(SEG_W),
                                    height: Val::Px(SEG_H),
                                    ..default()
                                },
                                BackgroundColor(vital_color(kind)),
                            ));
                        }
                        panel.spawn((
                            VitalValue(kind),
                            Text::new("100%"),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(254.0),
                                bottom: Val::Px(y + 1.0),
                                ..default()
                            },
                        ));
                    }
                });
        });
}

fn spawn_hotbar(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn((
            GameplayHud,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(14.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|bar| {
            for index in 0..9 {
                bar.spawn((
                    HotbarSlot(index),
                    Node {
                        width: Val::Px(44.0),
                        height: Val::Px(44.0),
                        position_type: PositionType::Relative,
                        padding: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(if index == 0 {
                        pixel_highlight()
                    } else {
                        pixel_border()
                    }),
                ))
                .with_children(|slot| {
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(3.0),
                            right: Val::Px(3.0),
                            top: Val::Px(3.0),
                            bottom: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.09, 0.085, 0.96)),
                    ));
                    slot.spawn((
                        HotbarIcon(index),
                        ImageNode::new(asset_server.load("sprites/item_scrap.png")),
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            left: Val::Px(7.0),
                            top: Val::Px(7.0),
                            width: Val::Px(30.0),
                            height: Val::Px(30.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        HotbarQty(index),
                        Text::new(""),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            right: Val::Px(4.0),
                            bottom: Val::Px(2.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        Text::new((index + 1).to_string()),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.87, 0.70)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(5.0),
                            top: Val::Px(2.0),
                            ..default()
                        },
                    ));
                });
            }
        });
}

fn spawn_field_log(commands: &mut Commands) {
    commands
        .spawn((
            FieldLogFrame,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(104.0),
                right: Val::Px(-238.0),
                width: Val::Px(282.0),
                min_height: Val::Px(64.0),
                padding: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(pixel_border()),
        ))
        .with_children(|outer| {
            outer
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(58.0),
                        padding: UiRect::all(Val::Px(9.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg()),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        FieldLogAccent,
                        Node {
                            width: Val::Px(30.0),
                            height: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(pixel_highlight()),
                    ));
                    inner.spawn((
                        FieldLogHeader,
                        Text::new("FIELD LOG"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.66, 0.54)),
                    ));
                    inner.spawn((
                        FieldLogText,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.97, 0.93, 0.80)),
                    ));
                });
        });
}

fn spawn_crafting_overlay(commands: &mut Commands) {
    commands
        .spawn((
            CraftingOverlay,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                CraftingText,
                Text::new("CRAFTING"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.88, 0.70)),
                Node {
                    width: Val::Px(560.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.055, 0.05, 0.99)),
            ));
        });
}

fn spawn_pause_overlay(commands: &mut Commands) {
    commands
        .spawn((
            PauseOverlay,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.01, 0.01, 0.88)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("PAUSED / QUICK HELP\n\nEsc  Resume\nG  Full walkthrough\nL  Session leaderboard\nR  Restart run\nQ  Quit\n\nA/D or arrows  Move\nSpace  Jump\n1-9  Select item\nF  Use item\nC  Craft\nE  Attack"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.91, 0.89, 0.82)),
                Node {
                    width: Val::Px(500.0),
                    padding: UiRect::all(Val::Px(26.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.065, 0.06, 0.99)),
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
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.76, 0.20, 0.20)),
            ));
            overlay.spawn((
                DeathText,
                Text::new(""),
                TextFont {
                    font_size: 17.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.79, 0.73)),
            ));
        });
}

fn assist_new_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    pause: Res<PauseMenuState>,
    crafting: Res<CraftingMenu>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut state: ResMut<UxNoticeState>,
    mut last: ResMut<LastEvent>,
) {
    if state.moved || pause.open || crafting.open || guide.open || leaderboard.open {
        return;
    }
    if keyboard.any_pressed([
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ]) {
        state.moved = true;
        return;
    }
    let unrelated = keyboard.any_just_pressed([
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::Space,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyC,
        KeyCode::Enter,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]);
    if unrelated && !state.movement_hint_sent {
        state.movement_hint_sent = true;
        last.show("GUIDE  MOVE WITH A / D OR ARROW KEYS  [G] WALKTHROUGH");
    }
}

fn emit_context_notices(
    inventory: Query<&PlayerInventory, With<Player>>,
    pause: Res<PauseMenuState>,
    crafting: Res<CraftingMenu>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut state: ResMut<UxNoticeState>,
    mut last: ResMut<LastEvent>,
) {
    if pause.open || crafting.open || guide.open || leaderboard.open {
        return;
    }
    if let Ok(inventory) = inventory.single() {
        let ready = first_craftable(&inventory.0).map(|kind| kind.label().to_uppercase());
        if ready != state.craft_ready {
            if let Some(name) = ready.as_ref() {
                if last.remaining <= 0.05 {
                    last.show(format!("CRAFT READY  {}  [C]", name));
                }
            }
            state.craft_ready = ready;
        }
    }
}

fn pause_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut pause: ResMut<PauseMenuState>,
    mut crafting: ResMut<CraftingMenu>,
    mut player_velocity: Query<&mut LinearVelocity, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !guide.open && !leaderboard.open {
        if crafting.open {
            crafting.open = false;
        } else {
            pause.open = !pause.open;
        }
    }

    if pause.open {
        if let Ok(mut velocity) = player_velocity.single_mut() {
            *velocity = LinearVelocity::ZERO;
        }
        if keyboard.just_pressed(KeyCode::KeyR) {
            pause.open = false;
        }
        if keyboard.just_pressed(KeyCode::KeyQ) {
            exit.write(AppExit::Success);
        }
    }
}

fn update_objective(stats: Res<RunStats>, mut text: Query<&mut Text, With<ObjectiveText>>) {
    if let Ok(mut text) = text.single_mut() {
        **text = if stats.cargo_recovered {
            "MISSION // 2 OF 2\nDELIVER CARGO TO EXTRACTION".to_string()
        } else {
            "MISSION // 1 OF 2\nDESCEND + RECOVER LOST CARGO".to_string()
        };
    }
}

fn update_layer_text(depth: Res<CurrentDepth>, mut text: Query<&mut Text, With<LayerText>>) {
    if let Ok(mut text) = text.single_mut() {
        **text = format!("LAYER {}", depth.0);
    }
}

fn update_vitals(
    player: Query<(&Body, &Survival), With<Player>>,
    mut segments: Query<(&VitalSegment, &mut BackgroundColor)>,
    mut values: Query<(&VitalValue, &mut Text)>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    let value_for = |kind: VitalKind| -> f32 {
        match kind {
            VitalKind::Health => body.0.blood_volume(),
            VitalKind::Hunger => survival.0.hunger(),
            VitalKind::Thirst => survival.0.thirst(),
        }
        .clamp(0.0, 1.0)
    };

    for (segment, mut bg) in &mut segments {
        let lit = (value_for(segment.kind) * 10.0).ceil() as usize;
        bg.0 = if segment.index < lit {
            vital_color(segment.kind)
        } else {
            Color::srgb(0.09, 0.08, 0.08)
        };
    }
    for (kind, mut text) in &mut values {
        **text = format!("{}%", (value_for(kind.0) * 100.0).round() as i32);
    }
}

fn update_hotbar(
    inventory: Query<&PlayerInventory, With<Player>>,
    selected: Res<SelectedSlot>,
    asset_server: Res<AssetServer>,
    mut slots: Query<(&HotbarSlot, &mut BackgroundColor)>,
    mut icons: Query<(&HotbarIcon, &mut Node, &mut ImageNode), Without<HotbarQty>>,
    mut quantities: Query<(&HotbarQty, &mut Node, &mut Text), Without<HotbarIcon>>,
) {
    let Ok(inventory) = inventory.single() else {
        return;
    };
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.0 == selected.0 {
            pixel_highlight()
        } else {
            pixel_border()
        };
    }
    for (slot, mut node, mut image) in &mut icons {
        if let Some(stack) = inventory.0.stacks().get(slot.0) {
            node.display = Display::Flex;
            image.image = asset_server.load(stack.kind.sprite_path());
        } else {
            node.display = Display::None;
        }
    }
    for (slot, mut node, mut text) in &mut quantities {
        if let Some(stack) = inventory.0.stacks().get(slot.0) {
            node.display = Display::Flex;
            **text = stack.quantity.to_string();
        } else {
            node.display = Display::None;
        }
    }
}

fn is_urgent_event(text: &str) -> bool {
    text.contains("POISON")
        || text.contains("DEHYDRATED")
        || text.contains("STARVING")
        || text.contains("TRAP")
        || text.contains("BLEED")
        || text.contains("ATTACK")
}

fn field_log_heading(text: &str) -> &'static str {
    if text.starts_with("GUIDE") {
        "GUIDE"
    } else if is_urgent_event(text) {
        "DANGER"
    } else if text.starts_with("PICKUP") {
        "DISCOVERED"
    } else if text.starts_with("CRAFT") {
        "CRAFTING"
    } else {
        "FIELD LOG"
    }
}

fn update_field_log(
    time: Res<Time>,
    last: Res<LastEvent>,
    announcement: Res<DepthAnnouncement>,
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    pause: Res<PauseMenuState>,
    crafting: Res<CraftingMenu>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut frame: Query<&mut Node, With<FieldLogFrame>>,
    mut header: Query<&mut Text, (With<FieldLogHeader>, Without<FieldLogText>)>,
    mut text: Query<&mut Text, (With<FieldLogText>, Without<FieldLogHeader>)>,
    mut accent: Query<&mut BackgroundColor, With<FieldLogAccent>>,
) {
    let Ok(mut frame) = frame.single_mut() else {
        return;
    };
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    if dead || stats.extracted || pause.open || crafting.open || guide.open || leaderboard.open {
        frame.display = Display::None;
        return;
    }

    let duplicate_objective =
        last.text.contains("OBJECTIVE") || last.text.contains("MISSION COMPLETE");
    let urgent = last.remaining > 0.0 && is_urgent_event(&last.text);

    let (remaining, total, heading, message) = if urgent && !duplicate_objective {
        (last.remaining, 2.6, field_log_heading(&last.text), last.text.clone())
    } else if announcement.remaining > 0.0 {
        (announcement.remaining, 2.5, "DEPTH", announcement.text.clone())
    } else if last.remaining > 0.0 && !duplicate_objective {
        (last.remaining, 2.6, field_log_heading(&last.text), last.text.clone())
    } else {
        frame.display = Display::None;
        return;
    };

    frame.display = Display::Flex;
    let enter = ((total - remaining) / 0.18).clamp(0.0, 1.0);
    let exit = (remaining / 0.24).clamp(0.0, 1.0);
    let visibility = enter.min(exit);
    let eased = 1.0 - (1.0 - visibility) * (1.0 - visibility);
    frame.right = Val::Px(-238.0 + 256.0 * eased);

    if let Ok(mut h) = header.single_mut() {
        **h = heading.to_string();
    }
    if let Ok(mut t) = text.single_mut() {
        **t = message;
    }
    if let Ok(mut color) = accent.single_mut() {
        let pulse = 0.78 + 0.22 * (time.elapsed_secs() * 9.0).sin().abs();
        color.0 = Color::srgb(0.80 * pulse, 0.67 * pulse, 0.34 * pulse);
    }
}

fn update_crafting_overlay(
    crafting: Res<CraftingMenu>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut overlay: Query<&mut Node, With<CraftingOverlay>>,
    mut text: Query<&mut Text, With<CraftingText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    let show = crafting.open && !guide.open && !leaderboard.open;
    overlay.display = if show { Display::Flex } else { Display::None };
    if show {
        if let Ok(mut text) = text.single_mut() {
            **text = format!(
                "CRAFTING\n\n{}\n\n1 / 2 / 3 craft     C or Esc close",
                recipe_descriptions().join("\n\n")
            );
        }
    }
}

fn update_pause_overlay(
    pause: Res<PauseMenuState>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut overlay: Query<&mut Node, With<PauseOverlay>>,
) {
    if let Ok(mut overlay) = overlay.single_mut() {
        overlay.display = if pause.open && !guide.open && !leaderboard.open {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn update_death_overlay(
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    cause: Res<LastDamageCause>,
    mut overlay: Query<&mut Node, With<DeathOverlay>>,
    mut text: Query<&mut Text, With<DeathText>>,
) {
    let Ok(body) = player.single() else {
        return;
    };
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    if body.0.is_dead() && !stats.extracted {
        overlay.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            let reason = match cause.0 {
                DamageCause::Trap => "Killed by a cave trap",
                DamageCause::Enemy => "Killed by a cave creature",
                DamageCause::Fall => {
                    if body.0.total_bleed_rate() > 0.0005 {
                        "Bled out after a hard fall"
                    } else {
                        "Killed by a severe fall"
                    }
                }
                DamageCause::Dehydration => "Died from severe dehydration",
                DamageCause::Unknown => "Died from blood loss",
            };
            **text = format!(
                "{}\nDeepest layer: {}    {:.0}s survived\n\nR  Restart",
                reason, stats.deepest_layer, stats.elapsed_secs
            );
        }
    } else {
        overlay.display = Display::None;
    }
}

fn enforce_gameplay_hud_visibility(
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    pause: Res<PauseMenuState>,
    crafting: Res<CraftingMenu>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut hud: Query<&mut Node, With<GameplayHud>>,
) {
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    let show = !(dead
        || stats.extracted
        || pause.open
        || crafting.open
        || guide.open
        || leaderboard.open);
    for mut node in &mut hud {
        node.display = if show { Display::Flex } else { Display::None };
    }
}

fn sync_overlay_pause(
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    pause: Res<PauseMenuState>,
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut velocity: Query<&mut LinearVelocity, With<Player>>,
) {
    if guide.open || leaderboard.open {
        crafting.open = false;
    }
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    let blocked = pause.open
        || crafting.open
        || guide.open
        || leaderboard.open
        || dead
        || stats.extracted;
    if blocked {
        virtual_time.pause();
        if let Ok(mut velocity) = velocity.single_mut() {
            *velocity = LinearVelocity::ZERO;
        }
    } else {
        virtual_time.unpause();
    }
}
