//! Beginner-first player UI.
//! The game always answers: what is my goal, what is selected, what does it
//! do, and what is hurting me. Detailed crafting/help live in overlays.

use avian2d::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::Body;
use crate::items::{
    recipe_descriptions, CraftingMenu, LastEvent, PlayerInventory, SelectedSlot,
};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{CurrentDepth, DepthAnnouncement, RunStats};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .init_resource::<PauseMenuState>()
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    pause_controls,
                    update_objective,
                    update_depth_and_pack,
                    update_vitals,
                    update_warning,
                    update_hotbar,
                    update_event_toast,
                    update_depth_toast,
                    update_tutorial,
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
struct TutorialState {
    step: u8,
    complete: bool,
}

#[derive(Component)]
struct GameplayHud;
#[derive(Component)]
struct ObjectiveText;
#[derive(Component)]
struct DepthText;
#[derive(Component)]
struct PackWeightText;
#[derive(Component)]
struct SelectedItemText;
#[derive(Component)]
struct WarningText;
#[derive(Component)]
struct EventToast;
#[derive(Component)]
struct DepthToast;
#[derive(Component)]
struct TutorialPanel;
#[derive(Component)]
struct TutorialText;
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
struct VitalText(VitalKind);
#[derive(Component)]
struct HotbarSlot(usize);
#[derive(Component)]
struct HotbarIcon(usize);
#[derive(Component)]
struct HotbarQty(usize);

fn panel_bg() -> Color {
    Color::srgba(0.035, 0.03, 0.03, 0.88)
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_objective(&mut commands);
    spawn_depth_and_pack(&mut commands);
    spawn_vitals(&mut commands, &asset_server);
    spawn_hotbar(&mut commands, &asset_server);
    spawn_toasts(&mut commands);
    spawn_tutorial(&mut commands);
    spawn_crafting_overlay(&mut commands);
    spawn_pause_overlay(&mut commands);
    spawn_death_overlay(&mut commands);
    spawn_win_overlay(&mut commands);
}

fn spawn_objective(commands: &mut Commands) {
    commands.spawn((
        GameplayHud,
        ObjectiveText,
        Text::new(
            "OBJECTIVE 1/2 - RECOVER LOST CARGO\nGo DOWN. Land on the wide ledges. Reach the bottom chamber.",
        ),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(0.97, 0.86, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(16.0),
            width: Val::Px(520.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));
}

fn spawn_depth_and_pack(commands: &mut Commands) {
    commands.spawn((
        GameplayHud,
        DepthText,
        Text::new("DEPTH: LAYER 0 / 4"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.84, 0.81, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(18.0),
            ..default()
        },
    ));

    // Pack weight lives in the top-right, completely separate from the
    // bottom vitals/hotbar so small window sizes cannot make them overlap.
    commands.spawn((
        GameplayHud,
        PackWeightText,
        Text::new("PACK: 0 / 12 WEIGHT"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.70, 0.68, 0.62)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(18.0),
            ..default()
        },
    ));
}

fn spawn_vitals(commands: &mut Commands, asset_server: &AssetServer) {
    const LEFT: f32 = 14.0;
    const BOTTOM: f32 = 14.0;
    const ROW_H: f32 = 23.0;

    commands.spawn((
        GameplayHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(LEFT),
            bottom: Val::Px(BOTTOM),
            width: Val::Px(174.0),
            height: Val::Px(104.0),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    let rows = [
        (VitalKind::Health, "sprites/icon_health.png", "HEALTH"),
        (VitalKind::Hunger, "sprites/icon_hunger.png", "HUNGER"),
        (VitalKind::Thirst, "sprites/icon_thirst.png", "THIRST"),
        (VitalKind::Stamina, "sprites/icon_stamina.png", "STAMINA"),
    ];

    for (index, (kind, icon, label)) in rows.into_iter().enumerate() {
        let bottom = BOTTOM + 7.0 + index as f32 * ROW_H;
        commands.spawn((
            GameplayHud,
            ImageNode::new(asset_server.load(icon)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LEFT + 7.0),
                bottom: Val::Px(bottom + 1.0),
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                ..default()
            },
        ));
        commands.spawn((
            GameplayHud,
            VitalText(kind),
            Text::new(format!("{label} 100%")),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.86, 0.80)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(LEFT + 32.0),
                bottom: Val::Px(bottom + 2.0),
                ..default()
            },
        ));
    }

    commands.spawn((
        WarningText,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.60, 0.38)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(LEFT),
            bottom: Val::Px(BOTTOM + 111.0),
            padding: UiRect::all(Val::Px(7.0)),
            ..default()
        },
        BackgroundColor(panel_bg()),
    ));
}

fn spawn_hotbar(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn((
            GameplayHud,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(12.0),
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
                        ..default()
                    },
                    BackgroundColor(if index == 0 {
                        Color::srgba(0.70, 0.57, 0.25, 0.98)
                    } else {
                        Color::srgba(0.12, 0.11, 0.10, 0.92)
                    }),
                ))
                .with_children(|slot| {
                    slot.spawn((
                        HotbarIcon(index),
                        ImageNode::new(asset_server.load("sprites/item_scrap.png")),
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            left: Val::Px(6.0),
                            top: Val::Px(6.0),
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        HotbarQty(index),
                        Text::new(""),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            bottom: Val::Px(1.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        Text::new(format!("{}", index + 1)),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.90, 0.86, 0.72)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(3.0),
                            top: Val::Px(1.0),
                            ..default()
                        },
                    ));
                });
            }
        });

    commands.spawn((
        GameplayHud,
        SelectedItemText,
        Text::new("[1] EMPTY - walk into a labelled item to pick it up"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.88, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(61.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

fn spawn_toasts(commands: &mut Commands) {
    commands.spawn((
        EventToast,
        Text::new(""),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.96, 0.92, 0.78)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(91.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));

    commands.spawn((
        DepthToast,
        Text::new(""),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.81, 0.68)),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(92.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}

fn spawn_tutorial(commands: &mut Commands) {
    commands
        .spawn((
            TutorialPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(95.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                TutorialText,
                Text::new(
                    "NEW PLAYER 1/5: Your mission is to go DOWN. Use A / D or arrows to move.",
                ),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.94, 0.78)),
                Node {
                    width: Val::Px(700.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(panel_bg()),
            ));
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.76)),
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
                    padding: UiRect::all(Val::Px(22.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.055, 0.05, 0.98)),
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
            BackgroundColor(Color::srgba(0.01, 0.01, 0.01, 0.86)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new(
                    "PAUSED\n\nEsc  Resume\nR    Restart run\nQ    Quit to desktop\n\nBASICS\nA/D or arrows  Move\nSpace          Jump\n1-9            Select hotbar slot\nF              Use selected item\nC              Open crafting\nE              Melee attack\n\nMISSION\nFollow the guaranteed ledges DOWN.\nRecover the cargo in the bottom chamber.\nThen cross the chamber to the green EXTRACTION pad.",
                ),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.91, 0.89, 0.82)),
                Node {
                    width: Val::Px(500.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.065, 0.06, 0.98)),
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
            if !pause.open {
                virtual_time.unpause();
            }
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
        "OBJECTIVE 2/2 - DELIVER THE CARGO\nCross the bottom chamber to the green EXTRACTION pad.".to_string()
    } else {
        "OBJECTIVE 1/2 - RECOVER LOST CARGO\nGo DOWN. Land on the wide ledges. Reach the bottom chamber.".to_string()
    };
}

fn update_depth_and_pack(
    depth: Res<CurrentDepth>,
    inventory: Query<&PlayerInventory, With<Player>>,
    mut depth_text: Query<&mut Text, (With<DepthText>, Without<PackWeightText>)>,
    mut pack_text: Query<&mut Text, (With<PackWeightText>, Without<DepthText>)>,
) {
    if let Ok(mut text) = depth_text.single_mut() {
        **text = format!("DEPTH: LAYER {} / 4", depth.0);
    }
    if let (Ok(inventory), Ok(mut text)) = (inventory.single(), pack_text.single_mut()) {
        **text = format!(
            "PACK: {} / {} WEIGHT",
            inventory.0.used_weight(),
            inventory.0.capacity()
        );
    }
}

fn update_vitals(
    player: Query<(&Body, &Survival), With<Player>>,
    mut texts: Query<(&VitalText, &mut Text)>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    for (kind, mut text) in &mut texts {
        let (label, value) = match kind.0 {
            VitalKind::Health => ("HEALTH", body.0.blood_volume()),
            VitalKind::Hunger => ("HUNGER", survival.0.hunger()),
            VitalKind::Thirst => ("THIRST", survival.0.thirst()),
            VitalKind::Stamina => ("STAMINA", survival.0.stamina()),
        };
        **text = format!(
            "{label} {:>3}%",
            (value.clamp(0.0, 1.0) * 100.0).round() as i32
        );
    }
}

fn update_warning(
    player: Query<(&Body, &Survival), With<Player>>,
    mut warning: Query<(&mut Node, &mut Text), With<WarningText>>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    let Ok((mut node, mut text)) = warning.single_mut() else {
        return;
    };

    let message = if body.0.total_bleed_rate() > 0.0 {
        Some("BLEEDING - select BANDAGE with 1-9, then press F")
    } else if body.0.has_untreated_leg_fracture() {
        Some("FRACTURE - select SPLINT with 1-9, then press F")
    } else if survival.0.thirst() < 0.22 {
        Some("VERY THIRSTY - select WATER with 1-9, then press F")
    } else if survival.0.hunger() < 0.22 {
        Some("VERY HUNGRY - select FOOD with 1-9, then press F")
    } else {
        None
    };

    if let Some(message) = message {
        node.display = Display::Flex;
        **text = message.to_string();
    } else {
        node.display = Display::None;
    }
}

fn update_hotbar(
    inventory: Query<&PlayerInventory, With<Player>>,
    selected: Res<SelectedSlot>,
    asset_server: Res<AssetServer>,
    mut slots: Query<(&HotbarSlot, &mut BackgroundColor)>,
    mut icons: Query<(&HotbarIcon, &mut Node, &mut ImageNode), Without<HotbarQty>>,
    mut quantities: Query<(&HotbarQty, &mut Node, &mut Text), Without<HotbarIcon>>,
    mut selected_text: Query<&mut Text, With<SelectedItemText>>,
) {
    let Ok(inventory) = inventory.single() else {
        return;
    };

    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.0 == selected.0 {
            Color::srgba(0.70, 0.57, 0.25, 0.98)
        } else {
            Color::srgba(0.12, 0.11, 0.10, 0.92)
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
            **text = format!("{}", stack.quantity);
        } else {
            node.display = Display::None;
        }
    }

    if let Ok(mut text) = selected_text.single_mut() {
        **text = if let Some(stack) = inventory.0.stacks().get(selected.0) {
            let action = if stack.kind.is_directly_usable() {
                "F = USE"
            } else {
                "C = CRAFT"
            };
            format!(
                "[{}] {} x{}  |  {}  |  {}",
                selected.0 + 1,
                stack.kind.label().to_uppercase(),
                stack.quantity,
                action,
                stack.kind.purpose()
            )
        } else {
            format!(
                "[{}] EMPTY - press another number or pick up a labelled item",
                selected.0 + 1
            )
        };
    }
}

fn update_event_toast(
    last: Res<LastEvent>,
    mut toast: Query<(&mut Node, &mut Text), With<EventToast>>,
) {
    let Ok((mut node, mut text)) = toast.single_mut() else {
        return;
    };
    if last.remaining > 0.0 {
        node.display = Display::Flex;
        **text = last.text.clone();
    } else {
        node.display = Display::None;
    }
}

fn update_depth_toast(
    announcement: Res<DepthAnnouncement>,
    mut toast: Query<(&mut Node, &mut Text), With<DepthToast>>,
) {
    let Ok((mut node, mut text)) = toast.single_mut() else {
        return;
    };
    if announcement.remaining > 0.0 {
        node.display = Display::Flex;
        **text = announcement.text.clone();
    } else {
        node.display = Display::None;
    }
}

fn update_tutorial(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Query<&PlayerInventory, With<Player>>,
    pause: Res<PauseMenuState>,
    crafting: Res<CraftingMenu>,
    mut tutorial: ResMut<TutorialState>,
    mut panel: Query<&mut Node, With<TutorialPanel>>,
    mut text: Query<&mut Text, With<TutorialText>>,
) {
    let Ok(mut panel) = panel.single_mut() else {
        return;
    };
    if tutorial.complete || pause.open || crafting.open {
        panel.display = Display::None;
        return;
    }
    panel.display = Display::Flex;

    match tutorial.step {
        0 => {
            if keyboard.any_pressed([
                KeyCode::KeyA,
                KeyCode::KeyD,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]) {
                tutorial.step = 1;
            }
        }
        1 => {
            if keyboard.just_pressed(KeyCode::Space) {
                tutorial.step = 2;
            }
        }
        2 => {
            if inventory
                .single()
                .map(|inventory| !inventory.0.stacks().is_empty())
                .unwrap_or(false)
            {
                tutorial.step = 3;
            }
        }
        3 => {
            if keyboard.any_just_pressed([
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
                KeyCode::KeyF,
            ]) {
                tutorial.step = 4;
            }
        }
        4 => {
            if keyboard.just_pressed(KeyCode::KeyC) {
                tutorial.complete = true;
            }
        }
        _ => tutorial.complete = true,
    }

    let message = match tutorial.step {
        0 => "NEW PLAYER 1/5: Your mission is to go DOWN. Use A / D or arrows to move.",
        1 => "NEW PLAYER 2/5: Press SPACE to jump. Falling is normal - aim for the wide ledges below.",
        2 => "NEW PLAYER 3/5: Items have names above them. Walk into one to pick it up.",
        3 => "NEW PLAYER 4/5: Press 1-9 to select a hotbar item. The selected item/action is shown above the hotbar.",
        _ => "NEW PLAYER 5/5: Press C to see simple recipes. The top-left objective always tells you what to do next.",
    };
    if let Ok(mut text) = text.single_mut() {
        **text = message.to_string();
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
            let recipes = recipe_descriptions().join("\n\n");
            **text = format!(
                "CRAFTING - choose what you actually need\n\n{recipes}\n\nPress 1, 2 or 3 to craft that recipe. C or Esc closes this screen."
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
            let cause = if body.0.total_bleed_rate() > 0.0 {
                "Cause: untreated bleeding. Next run: use a BANDAGE sooner."
            } else {
                "Cause: catastrophic trauma / blood loss. Next run: control long falls."
            };
            **text = format!(
                "{cause}\nDeepest layer: {}\nSurvived: {:.0}s\n\nPress R to restart",
                stats.deepest_layer, stats.elapsed_secs
            );
        }
    } else {
        overlay.display = Display::None;
    }
}

fn update_win_overlay(
    stats: Res<RunStats>,
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
                "Cargo recovered and delivered to extraction.\nDeepest layer: {}\nTime: {:.0}s\n\nPress R to run again",
                stats.deepest_layer, stats.elapsed_secs
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
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    let show = !(dead || stats.extracted || pause.open || crafting.open);
    for mut node in &mut hud {
        node.display = if show { Display::Flex } else { Display::None };
    }
}
