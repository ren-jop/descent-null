//! Compact retro-pixel HUD. Persistent UI is limited to one mission panel,
//! one centred layer label, three survival vitals and the hotbar. Everything
//! else goes through a single animated right-side field log.

use std::fs;

use avian2d::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::{Body, DamageCause, LastDamageCause};
use crate::items::{
    first_craftable, recipe_descriptions, CraftingMenu, LastEvent, PlayerInventory, SelectedSlot,
};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{BestTimes, CurrentDepth, DepthAnnouncement, RunStats, SessionTimes};

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
                    update_win_overlay,
                    enforce_gameplay_hud_visibility,
                )
                    .chain(),
            );
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
    thirst_band: u8,
    hunger_band: u8,
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
#[derive(Component)]
struct WinOverlay;
#[derive(Component)]
struct WinText;

#[derive(Component, Clone, Copy, PartialEq)]
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
    spawn_win_overlay(&mut commands);
}

fn spawn_objective(commands: &mut Commands) {
    // Border and text are separate entities so the inner panel can never cover
    // the objective text (the source of the previous top-left rendering bug).
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
    // A full-width parent does the centring; the text itself is a child. This
    // avoids relying on text alignment APIs and keeps the label truly centred.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
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
    const LEFT: f32 = 14.0;
    const BOTTOM: f32 = 14.0;
    const PANEL_W: f32 = 302.0;
    const PANEL_H: f32 = 112.0;
    const BAR_X: f32 = 102.0;
    const SEG_W: f32 = 13.0;
    const SEG_GAP: f32 = 2.0;
    const SEG_H: f32 = 16.0;
    const ROW_H: f32 = 30.0;

    commands.spawn((
        GameplayHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(LEFT),
            bottom: Val::Px(BOTTOM),
            width: Val::Px(PANEL_W),
            height: Val::Px(PANEL_H),
            ..default()
        },
        BackgroundColor(pixel_border()),
    ));
    commands.spawn((
        GameplayHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(LEFT + 3.0),
            bottom: Val::Px(BOTTOM + 3.0),
            width: Val::Px(PANEL_W - 6.0),
            height: Val::Px(PANEL_H - 6.0),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    let rows = [
        (VitalKind::Health, "HEALTH"),
        (VitalKind::Hunger, "HUNGER"),
        (VitalKind::Thirst, "THIRST"),
    ];

    for (row, (kind, label)) in rows.into_iter().enumerate() {
        let y = BOTTOM + PANEL_H - 31.0 - row as f32 * ROW_H;
        commands.spawn((
            GameplayHud,
            ImageNode::new(asset_server.load(vital_icon(kind))),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LEFT + 11.0),
                bottom: Val::Px(y - 1.0),
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                ..default()
            },
        ));
        commands.spawn((
            GameplayHud,
            Text::new(label),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.84, 0.82, 0.76)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LEFT + 38.0),
                bottom: Val::Px(y + 1.0),
                ..default()
            },
        ));

        for index in 0..10 {
            commands.spawn((
                GameplayHud,
                VitalSegment { kind, index },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(LEFT + BAR_X + index as f32 * (SEG_W + SEG_GAP)),
                    bottom: Val::Px(y),
                    width: Val::Px(SEG_W),
                    height: Val::Px(SEG_H),
                    ..default()
                },
                BackgroundColor(vital_color(kind)),
            ));
        }

        commands.spawn((
            GameplayHud,
            VitalValue(kind),
            Text::new("100%"),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LEFT + 257.0),
                bottom: Val::Px(y + 1.0),
                ..default()
            },
        ));
    }
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
            GameplayHud,
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
                Text::new("PAUSED / HELP\n\nEsc  Resume\nR  Restart run\nQ  Quit\n\nA/D or arrows  Move\nSpace  Jump\n1-9  Select item\nF  Use item\nC  Craft\nE  Attack\n\nMISSION\nDescend, recover the cargo, reach extraction."),
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
            BackgroundColor(Color::srgba(0.01, 0.03, 0.015, 0.96)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("MISSION COMPLETE"),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(Color::srgb(0.48, 0.82, 0.44)),
            ));
            overlay.spawn((
                WinText,
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
    mut state: ResMut<UxNoticeState>,
    mut last: ResMut<LastEvent>,
) {
    if state.moved {
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
        KeyCode::KeyQ,
        KeyCode::KeyR,
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
        last.show("GUIDE  MOVE WITH A / D OR ARROW KEYS");
    }
}

fn survival_band(value: f32, warning: f32, critical: f32) -> u8 {
    if value <= critical {
        2
    } else if value <= warning {
        1
    } else {
        0
    }
}

fn emit_context_notices(
    player: Query<&Survival, With<Player>>,
    inventory: Query<&PlayerInventory, With<Player>>,
    mut state: ResMut<UxNoticeState>,
    mut last: ResMut<LastEvent>,
) {
    let Ok(survival) = player.single() else {
        return;
    };

    let thirst = survival_band(survival.0.thirst(), 0.35, 0.12);
    let hunger = survival_band(survival.0.hunger(), 0.35, 0.10);

    if thirst > state.thirst_band {
        if thirst == 2 {
            last.show("DEHYDRATED  HEALTH IS FALLING - DRINK WATER");
        } else if last.remaining <= 0.05 {
            last.show("THIRST LOW  FIND WATER SOON");
        }
    }
    if hunger > state.hunger_band && last.remaining <= 0.05 {
        if hunger == 2 {
            last.show("STARVING  MOVEMENT HEAVILY REDUCED - EAT FOOD");
        } else {
            last.show("HUNGER LOW  MOVEMENT WILL SLOW");
        }
    }
    state.thirst_band = thirst;
    state.hunger_band = hunger;

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
    mut pause: ResMut<PauseMenuState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut player_velocity: Query<&mut LinearVelocity, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if crafting.open {
            crafting.open = false;
            return;
        }
        pause.open = !pause.open;
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

    if pause.open || crafting.open {
        virtual_time.pause();
    } else {
        virtual_time.unpause();
    }
}

fn update_objective(stats: Res<RunStats>, mut text: Query<&mut Text, With<ObjectiveText>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    **text = if stats.cargo_recovered {
        "MISSION // 2 OF 2\nDELIVER CARGO TO EXTRACTION".to_string()
    } else {
        "MISSION // 1 OF 2\nDESCEND + RECOVER LOST CARGO".to_string()
    };
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

fn update_field_log(
    time: Res<Time>,
    last: Res<LastEvent>,
    announcement: Res<DepthAnnouncement>,
    mut frame: Query<&mut Node, With<FieldLogFrame>>,
    mut header: Query<&mut Text, (With<FieldLogHeader>, Without<FieldLogText>)>,
    mut text: Query<&mut Text, (With<FieldLogText>, Without<FieldLogHeader>)>,
    mut accent: Query<&mut BackgroundColor, With<FieldLogAccent>>,
) {
    let Ok(mut frame) = frame.single_mut() else {
        return;
    };

    let last_is_duplicate_objective = last.text.contains("OBJECTIVE") || last.text.contains("MISSION COMPLETE");

    let (remaining, total, heading, message) = if announcement.remaining > 0.0 {
        (announcement.remaining, 2.5, "DEPTH", announcement.text.clone())
    } else if last.remaining > 0.0 && !last_is_duplicate_objective {
        let heading = if last.text.starts_with("GUIDE") {
            "GUIDE"
        } else if last.text.contains("DEHYDRATED")
            || last.text.contains("STARVING")
            || last.text.contains("TRAP")
            || last.text.contains("BLEED")
        {
            "DANGER"
        } else if last.text.starts_with("PICKUP") {
            "DISCOVERED"
        } else if last.text.starts_with("CRAFT") {
            "CRAFTING"
        } else {
            "FIELD LOG"
        };
        (last.remaining, 2.6, heading, last.text.clone())
    } else {
        frame.display = Display::None;
        return;
    };

    frame.display = Display::Flex;
    let enter = ((total - remaining) / 0.18).clamp(0.0, 1.0);
    let exit = (remaining / 0.24).clamp(0.0, 1.0);
    let visibility = enter.min(exit);
    let eased = 1.0 - (1.0 - visibility) * (1.0 - visibility);
    // Most of the card lives outside the window when collapsed, making the
    // field log feel like a compact instrument instead of a permanent panel.
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
    mut overlay: Query<&mut Node, With<CraftingOverlay>>,
    mut text: Query<&mut Text, With<CraftingText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    overlay.display = if crafting.open {
        Display::Flex
    } else {
        Display::None
    };
    if crafting.open {
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
    mut overlay: Query<&mut Node, With<PauseOverlay>>,
) {
    if let Ok(mut overlay) = overlay.single_mut() {
        overlay.display = if pause.open {
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
                DamageCause::Enemy => "Killed by a crawler attack",
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

fn update_win_overlay(
    stats: Res<RunStats>,
    best: Res<BestTimes>,
    session: Res<SessionTimes>,
    mut overlay: Query<&mut Node, With<WinOverlay>>,
    mut text: Query<&mut Text, With<WinText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    if stats.extracted {
        overlay.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            **text = format!(
                "RUN TIME  {:.1}s\n\nTHIS SESSION\n{}\n\nPERSONAL BESTS\n{}\n\nR  Run again",
                stats.elapsed_secs,
                session.formatted(),
                best.formatted()
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
    mut hud: Query<&mut Node, With<GameplayHud>>,
) {
    let dead = player
        .single()
        .map(|body| body.0.is_dead())
        .unwrap_or(false);
    let show = !(dead || stats.extracted || pause.open || crafting.open);
    for mut node in &mut hud {
        node.display = if show {
            Display::Flex
        } else {
            Display::None
        };
    }
}