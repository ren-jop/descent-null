//! Cohesive retro-pixel HUD and onboarding.
//! Normal play shows one objective, one always-current layer label, pixel vitals,
//! the hotbar, and one animated right-side field-log notification. Early-game
//! teaching is modal and deliberately interrupts play at the moment a new action
//! matters instead of leaving permanent guide prose on screen.

use std::fs;

use avian2d::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::{Body, DamageCause, LastDamageCause};
use crate::items::{
    first_craftable, recipe_descriptions, CraftingMenu, LastEvent, Pickup,
    PlayerInventory, SelectedSlot,
};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{BestTimes, CurrentDepth, DepthAnnouncement, RunStats, SessionTimes};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseMenuState>()
            .init_resource::<TutorialInterrupt>()
            .init_resource::<UiFont>()
            .add_systems(PreStartup, load_ui_font)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    apply_ui_font,
                    update_tutorial_interrupt,
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

#[derive(Resource)]
struct TutorialInterrupt {
    stage: u8,
    open: bool,
    start_x: Option<f32>,
}

impl Default for TutorialInterrupt {
    fn default() -> Self {
        Self {
            stage: 0,
            open: true,
            start_x: None,
        }
    }
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
struct TutorialOverlay;
#[derive(Component)]
struct TutorialTitle;
#[derive(Component)]
struct TutorialText;
#[derive(Component)]
struct TutorialPrompt;
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
    Stamina,
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
    Color::srgba(0.022, 0.020, 0.020, 0.94)
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
        VitalKind::Stamina => Color::srgb(0.68, 0.76, 0.20),
    }
}

fn vital_icon(kind: VitalKind) -> &'static str {
    match kind {
        VitalKind::Health => "sprites/icon_health.png",
        VitalKind::Hunger => "sprites/icon_hunger.png",
        VitalKind::Thirst => "sprites/icon_thirst.png",
        VitalKind::Stamina => "sprites/icon_stamina.png",
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
    spawn_tutorial_overlay(&mut commands);
    spawn_crafting_overlay(&mut commands);
    spawn_pause_overlay(&mut commands);
    spawn_death_overlay(&mut commands);
    spawn_win_overlay(&mut commands);
}

fn spawn_objective(commands: &mut Commands) {
    // The one and only persistent objective surface.
    commands.spawn((
        GameplayHud,
        ObjectiveText,
        Text::new("OBJECTIVE 1/2\nRECOVER THE LOST CARGO"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.87, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            width: Val::Px(410.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(pixel_border()),
    )).with_children(|outer| {
        outer.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(3.0),
                bottom: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(panel_bg()),
        ));
    });
}

fn spawn_layer_label(commands: &mut Commands) {
    // Deliberately not GameplayHud: it remains authoritative through death,
    // pause and respawn instead of being hidden/stuck like the old toast.
    commands.spawn((
        LayerText,
        Text::new("LAYER 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

fn spawn_vitals(commands: &mut Commands, asset_server: &AssetServer) {
    const LEFT: f32 = 14.0;
    const BOTTOM: f32 = 14.0;
    const PANEL_W: f32 = 302.0;
    const PANEL_H: f32 = 142.0;
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
        (VitalKind::Stamina, "STAMINA"),
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
    commands.spawn((
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
    )).with_children(|bar| {
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
            )).with_children(|slot| {
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
    // One notification surface only. Chunky nested rectangles give it a
    // pixel-frame look; update_field_log handles the slide/pulse transition.
    commands.spawn((
        GameplayHud,
        FieldLogFrame,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(112.0),
            right: Val::Px(-70.0),
            width: Val::Px(342.0),
            min_height: Val::Px(84.0),
            padding: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(pixel_border()),
    )).with_children(|outer| {
        outer.spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(78.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(panel_bg()),
        )).with_children(|inner| {
            inner.spawn((
                FieldLogAccent,
                Node {
                    width: Val::Px(38.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(pixel_highlight()),
            ));
            inner.spawn((
                FieldLogHeader,
                Text::new("FIELD LOG"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.66, 0.54)),
            ));
            inner.spawn((
                FieldLogText,
                Text::new(""),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.97, 0.93, 0.80)),
            ));
        });
    });
}

fn spawn_tutorial_overlay(commands: &mut Commands) {
    commands.spawn((
        TutorialOverlay,
        Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
    )).with_children(|screen| {
        screen.spawn((
            Node {
                width: Val::Px(720.0),
                min_height: Val::Px(260.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.72, 0.59, 0.28)),
        )).with_children(|outer| {
            outer.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(250.0),
                    padding: UiRect::all(Val::Px(22.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.055, 0.05, 0.045)),
            )).with_children(|inner| {
                inner.spawn((
                    TutorialTitle,
                    Text::new("FIELD MANUAL // MOVEMENT"),
                    TextFont {
                        font_size: 17.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.77, 0.34)),
                ));
                inner.spawn((
                    TutorialText,
                    Text::new("Your mission is below. Move left and right to line up safe drops."),
                    TextFont {
                        font_size: 23.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.96, 0.94, 0.88)),
                ));
                inner.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.26, 0.23, 0.18)),
                ));
                inner.spawn((
                    TutorialPrompt,
                    Text::new("PRESS  A  OR  D  TO CONTINUE"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn spawn_crafting_overlay(commands: &mut Commands) {
    commands.spawn((
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
    )).with_children(|overlay| {
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
    commands.spawn((
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
    )).with_children(|overlay| {
        overlay.spawn((
            Text::new("PAUSED\n\nEsc  Resume\nR  Restart run\nQ  Quit\n\nA/D  Move     Space  Jump\n1-9  Select   F  Use\nC  Craft      E  Attack\n\nMISSION\nDescend, recover the cargo, reach extraction."),
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
    commands.spawn((
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
    )).with_children(|overlay| {
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
    commands.spawn((
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
    )).with_children(|overlay| {
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

fn tutorial_copy(stage: u8, craft_name: Option<&str>) -> (&'static str, String, String) {
    match stage {
        0 => (
            "FIELD MANUAL // MOVEMENT",
            "Your mission is below. Move left and right to line up safe drops.".to_string(),
            "PRESS  A  OR  D  TO CONTINUE".to_string(),
        ),
        1 => (
            "FIELD MANUAL // JUMPING",
            "Jump to correct your position before a drop. Short controlled falls are part of the route.".to_string(),
            "PRESS  SPACE  TO CONTINUE".to_string(),
        ),
        2 => (
            "FIELD MANUAL // SUPPLIES",
            "A labelled supply is close. Walk into world items to collect them automatically.".to_string(),
            "PRESS  ENTER  TO CONTINUE".to_string(),
        ),
        3 => (
            "FIELD MANUAL // HOTBAR",
            "Collected items appear in the hotbar. The gold-framed slot is the item you are holding.".to_string(),
            "PRESS  1-9  TO SELECT A SLOT".to_string(),
        ),
        4 => (
            "FIELD MANUAL // USING ITEMS",
            "Usable supplies are activated from the selected hotbar slot. Treatment and food only affect the real simulation state.".to_string(),
            "PRESS  F  TO CONTINUE".to_string(),
        ),
        _ => (
            "FIELD MANUAL // CRAFTING",
            format!(
                "You now have the materials for {}. Craft only when you need the result.",
                craft_name.unwrap_or("a recipe").to_uppercase()
            ),
            "PRESS  C  TO OPEN CRAFTING".to_string(),
        ),
    }
}

fn update_tutorial_interrupt(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    inventory: Query<&PlayerInventory, With<Player>>,
    pickups: Query<&Transform, With<Pickup>>,
    crafting: Res<CraftingMenu>,
    pause: Res<PauseMenuState>,
    mut tutorial: ResMut<TutorialInterrupt>,
    mut overlay: Query<&mut Node, With<TutorialOverlay>>,
    mut title: Query<&mut Text, With<TutorialTitle>>,
    mut body: Query<&mut Text, (With<TutorialText>, Without<TutorialTitle>, Without<TutorialPrompt>)>,
    mut prompt: Query<&mut Text, (With<TutorialPrompt>, Without<TutorialTitle>, Without<TutorialText>)>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        overlay.display = Display::None;
        return;
    };
    if tutorial.start_x.is_none() {
        tutorial.start_x = Some(player_transform.translation.x);
    }

    // Trigger the next modal only when its action is actually relevant.
    if !tutorial.open {
        match tutorial.stage {
            1 => {
                let start = tutorial.start_x.unwrap_or(player_transform.translation.x);
                if (player_transform.translation.x - start).abs() > 55.0 {
                    tutorial.open = true;
                }
            }
            2 => {
                let p = player_transform.translation.truncate();
                if pickups.iter().any(|t| p.distance(t.translation.truncate()) < 120.0) {
                    tutorial.open = true;
                }
            }
            3 => {
                if inventory
                    .single()
                    .map(|inv| !inv.0.stacks().is_empty())
                    .unwrap_or(false)
                {
                    tutorial.open = true;
                }
            }
            4 => {
                tutorial.open = true;
            }
            5 => {
                if inventory
                    .single()
                    .ok()
                    .and_then(|inv| first_craftable(&inv.0))
                    .is_some()
                {
                    tutorial.open = true;
                }
            }
            _ => {}
        }
    }

    if tutorial.stage >= 6 {
        tutorial.open = false;
    }

    if tutorial.open && !pause.open && !crafting.open {
        let craft_name = inventory
            .single()
            .ok()
            .and_then(|inv| first_craftable(&inv.0))
            .map(|kind| kind.label());
        let (heading, copy, action) = tutorial_copy(tutorial.stage, craft_name);
        if let Ok(mut text) = title.single_mut() {
            **text = heading.to_string();
        }
        if let Ok(mut text) = body.single_mut() {
            **text = copy;
        }
        if let Ok(mut text) = prompt.single_mut() {
            **text = action;
        }
        overlay.display = Display::Flex;

        let accepted = match tutorial.stage {
            0 => keyboard.any_just_pressed([
                KeyCode::KeyA,
                KeyCode::KeyD,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]),
            1 => keyboard.just_pressed(KeyCode::Space),
            2 => keyboard.just_pressed(KeyCode::Enter),
            3 => keyboard.any_just_pressed([
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
            ]),
            4 => keyboard.just_pressed(KeyCode::KeyF),
            5 => keyboard.just_pressed(KeyCode::KeyC),
            _ => false,
        };
        if accepted {
            tutorial.open = false;
            tutorial.stage += 1;
        }
    } else {
        overlay.display = Display::None;
    }
}

fn pause_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    tutorial: Res<TutorialInterrupt>,
    mut pause: ResMut<PauseMenuState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut player_velocity: Query<&mut LinearVelocity, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !tutorial.open {
        if crafting.open {
            crafting.open = false;
            return;
        }
        pause.open = !pause.open;
    }

    if pause.open || tutorial.open {
        if let Ok(mut velocity) = player_velocity.single_mut() {
            *velocity = LinearVelocity::ZERO;
        }
    }
    if pause.open {
        if keyboard.just_pressed(KeyCode::KeyR) {
            pause.open = false;
        }
        if keyboard.just_pressed(KeyCode::KeyQ) {
            exit.write(AppExit::Success);
        }
    }

    if pause.open || crafting.open || tutorial.open {
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
        "OBJECTIVE 2/2\nDELIVER CARGO TO EXTRACTION".to_string()
    } else {
        "OBJECTIVE 1/2\nRECOVER THE LOST CARGO".to_string()
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
            VitalKind::Stamina => survival.0.stamina(),
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

    let (remaining, total, heading, message) = if announcement.remaining > 0.0 {
        (
            announcement.remaining,
            2.5,
            "DEPTH REACHED",
            announcement.text.clone(),
        )
    } else if last.remaining > 0.0 {
        let heading = if last.text.contains("TRAP") || last.text.contains("BLEED") {
            "DANGER"
        } else if last.text.starts_with("PICKUP") {
            "DISCOVERED"
        } else if last.text.starts_with("CRAFTED") {
            "CRAFTED"
        } else if last.text.contains("MISSION") || last.text.contains("OBJECTIVE") {
            "MISSION"
        } else {
            "FIELD LOG"
        };
        (last.remaining, 2.6, heading, last.text.clone())
    } else {
        frame.display = Display::None;
        return;
    };

    frame.display = Display::Flex;
    let enter = ((total - remaining) / 0.22).clamp(0.0, 1.0);
    let exit = (remaining / 0.30).clamp(0.0, 1.0);
    let visibility = enter.min(exit);
    let eased = 1.0 - (1.0 - visibility) * (1.0 - visibility);
    frame.right = Val::Px(-74.0 + 92.0 * eased);

    if let Ok(mut h) = header.single_mut() {
        **h = heading.to_string();
    }
    if let Ok(mut t) = text.single_mut() {
        **t = message;
    }
    if let Ok(mut color) = accent.single_mut() {
        let pulse = 0.75 + 0.25 * (time.elapsed_secs() * 10.0).sin().abs();
        color.0 = Color::srgb(0.80 * pulse, 0.67 * pulse, 0.34 * pulse);
    }
}

fn update_crafting_overlay(
    crafting: Res<CraftingMenu>,
    tutorial: Res<TutorialInterrupt>,
    mut overlay: Query<&mut Node, With<CraftingOverlay>>,
    mut text: Query<&mut Text, With<CraftingText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    overlay.display = if crafting.open && !tutorial.open {
        Display::Flex
    } else {
        Display::None
    };
    if crafting.open && !tutorial.open {
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
    tutorial: Res<TutorialInterrupt>,
    mut hud: Query<&mut Node, With<GameplayHud>>,
) {
    let dead = player
        .single()
        .map(|body| body.0.is_dead())
        .unwrap_or(false);
    let show = !(dead || stats.extracted || pause.open || crafting.open || tutorial.open);
    for mut node in &mut hud {
        node.display = if show {
            Display::Flex
        } else {
            Display::None
        };
    }
}
