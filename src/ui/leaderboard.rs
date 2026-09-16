use bevy::prelude::*;

use crate::body::Body;
use crate::player::Player;
use crate::world::{RunStats, SessionTimes};

#[derive(Resource, Default)]
pub struct LeaderboardState {
    pub open: bool,
    pub name_entry: bool,
    pub name_buffer: String,
    handled_extraction: bool,
}

#[derive(Component)]
struct MiniLeaderboard;
#[derive(Component)]
struct MiniLeaderboardButton;
#[derive(Component)]
struct MiniLeaderboardText;
#[derive(Component)]
struct FullLeaderboard;
#[derive(Component)]
struct FullLeaderboardButton;
#[derive(Component)]
struct FullLeaderboardText;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderboardState>()
            .add_systems(PreUpdate, consume_leaderboard_input)
            .add_systems(Startup, spawn_leaderboard)
            .add_systems(
                Update,
                (
                    begin_name_entry_if_needed,
                    leaderboard_controls,
                    update_mini_leaderboard,
                    update_full_leaderboard,
                )
                    .chain(),
            );
    }
}

fn panel_bg() -> Color {
    Color::srgba(0.022, 0.020, 0.020, 0.96)
}
fn pixel_border() -> Color {
    Color::srgb(0.43, 0.39, 0.29)
}
fn pixel_highlight() -> Color {
    Color::srgb(0.80, 0.67, 0.34)
}

fn best_line(session: &SessionTimes) -> String {
    session
        .best()
        .map(|record| format!("{}  {:.1}s", record.name, record.seconds))
        .unwrap_or_else(|| "--".to_string())
}

fn spawn_leaderboard(mut commands: Commands) {
    commands
        .spawn((
            MiniLeaderboard,
            Button,
            MiniLeaderboardButton,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                bottom: Val::Px(14.0),
                width: Val::Px(205.0),
                min_height: Val::Px(74.0),
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
                        min_height: Val::Px(68.0),
                        padding: UiRect::all(Val::Px(9.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg()),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        Text::new("SESSION RECORDS  [L]"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(pixel_highlight()),
                    ));
                    inner.spawn((
                        MiniLeaderboardText,
                        Text::new("BEST  --"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.94, 0.91, 0.82)),
                    ));
                    inner.spawn((
                        Text::new("CLICK / L  EXPAND"),
                        TextFont {
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.60, 0.57, 0.50)),
                    ));
                });
        });

    commands
        .spawn((
            FullLeaderboard,
            FullLeaderboardButton,
            Button,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.90)),
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        width: Val::Px(670.0),
                        min_height: Val::Px(440.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(pixel_highlight()),
                ))
                .with_children(|outer| {
                    outer
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(430.0),
                                padding: UiRect::all(Val::Px(28.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(18.0),
                                ..default()
                            },
                            BackgroundColor(panel_bg()),
                        ))
                        .with_children(|inner| {
                            inner.spawn((
                                Text::new("EXPO SESSION LEADERBOARD"),
                                TextFont {
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            inner.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(4.0),
                                    ..default()
                                },
                                BackgroundColor(pixel_highlight()),
                            ));
                            inner.spawn((
                                FullLeaderboardText,
                                Text::new("No completed runs yet"),
                                TextFont {
                                    font_size: 19.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.94, 0.91, 0.82)),
                            ));
                            inner.spawn((
                                Text::new("L / CLICK  CLOSE"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.66, 0.62, 0.52)),
                            ));
                        });
                });
        });
}

fn begin_name_entry_if_needed(
    stats: Res<RunStats>,
    session: Res<SessionTimes>,
    mut state: ResMut<LeaderboardState>,
) {
    if !stats.extracted {
        if state.handled_extraction {
            state.handled_extraction = false;
            state.name_entry = false;
            state.name_buffer.clear();
            state.open = false;
        }
        return;
    }
    if state.handled_extraction {
        return;
    }

    state.handled_extraction = true;
    state.open = true;
    if session.qualifies(stats.elapsed_secs) {
        state.name_entry = true;
        state.name_buffer.clear();
    }
}

fn key_to_char(keyboard: &ButtonInput<KeyCode>) -> Option<(KeyCode, char)> {
    let letters = [
        (KeyCode::KeyA, 'A'),
        (KeyCode::KeyB, 'B'),
        (KeyCode::KeyC, 'C'),
        (KeyCode::KeyD, 'D'),
        (KeyCode::KeyE, 'E'),
        (KeyCode::KeyF, 'F'),
        (KeyCode::KeyG, 'G'),
        (KeyCode::KeyH, 'H'),
        (KeyCode::KeyI, 'I'),
        (KeyCode::KeyJ, 'J'),
        (KeyCode::KeyK, 'K'),
        (KeyCode::KeyL, 'L'),
        (KeyCode::KeyM, 'M'),
        (KeyCode::KeyN, 'N'),
        (KeyCode::KeyO, 'O'),
        (KeyCode::KeyP, 'P'),
        (KeyCode::KeyQ, 'Q'),
        (KeyCode::KeyR, 'R'),
        (KeyCode::KeyS, 'S'),
        (KeyCode::KeyT, 'T'),
        (KeyCode::KeyU, 'U'),
        (KeyCode::KeyV, 'V'),
        (KeyCode::KeyW, 'W'),
        (KeyCode::KeyX, 'X'),
        (KeyCode::KeyY, 'Y'),
        (KeyCode::KeyZ, 'Z'),
    ];
    for (key, ch) in letters {
        if keyboard.just_pressed(key) {
            return Some((key, ch));
        }
    }
    let digits = [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ];
    for (key, ch) in digits {
        if keyboard.just_pressed(key) {
            return Some((key, ch));
        }
    }
    None
}

fn consume_leaderboard_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    stats: Res<RunStats>,
    mut session: ResMut<SessionTimes>,
    mut state: ResMut<LeaderboardState>,
) {
    // A mid-run records screen is informational. Do not let R leak through
    // and silently restart the cave while the player is viewing it.
    if state.open && !stats.extracted && !state.name_entry {
        if keyboard.just_pressed(KeyCode::KeyR) {
            keyboard.clear_just_pressed(KeyCode::KeyR);
        }
        return;
    }

    if !state.name_entry {
        return;
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        state.name_buffer.pop();
        keyboard.clear_just_pressed(KeyCode::Backspace);
        return;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        if state.name_buffer.len() < 12 {
            state.name_buffer.push(' ');
        }
        keyboard.clear_just_pressed(KeyCode::Space);
        return;
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        let name = state.name_buffer.trim().to_string();
        session.record(name, stats.elapsed_secs);
        state.name_entry = false;
        keyboard.clear_just_pressed(KeyCode::Enter);
        return;
    }
    if let Some((key, ch)) = key_to_char(&keyboard) {
        if state.name_buffer.len() < 12 {
            state.name_buffer.push(ch);
        }
        keyboard.clear_just_pressed(key);
    }
}

fn leaderboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    stats: Res<RunStats>,
    mut state: ResMut<LeaderboardState>,
    mini_interaction: Query<&Interaction, (Changed<Interaction>, With<MiniLeaderboardButton>)>,
    full_interaction: Query<&Interaction, (Changed<Interaction>, With<FullLeaderboardButton>)>,
) {
    if state.name_entry || stats.extracted {
        return;
    }

    let mini_clicked = mini_interaction
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let full_clicked = full_interaction
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if keyboard.just_pressed(KeyCode::KeyL) || mini_clicked {
        state.open = !state.open;
    } else if state.open && full_clicked {
        state.open = false;
    }
}

fn update_mini_leaderboard(
    session: Res<SessionTimes>,
    state: Res<LeaderboardState>,
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    mut frame: Query<&mut Node, With<MiniLeaderboard>>,
    mut text: Query<&mut Text, With<MiniLeaderboardText>>,
) {
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    if let Ok(mut frame) = frame.single_mut() {
        frame.display = if state.open || dead || stats.extracted {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut text) = text.single_mut() {
        **text = format!("BEST  {}", best_line(&session));
    }
}

fn update_full_leaderboard(
    session: Res<SessionTimes>,
    stats: Res<RunStats>,
    state: Res<LeaderboardState>,
    mut frame: Query<&mut Node, With<FullLeaderboard>>,
    mut text: Query<&mut Text, With<FullLeaderboardText>>,
) {
    if let Ok(mut frame) = frame.single_mut() {
        frame.display = if state.open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !state.open {
        return;
    }

    if let Ok(mut text) = text.single_mut() {
        **text = if state.name_entry {
            format!(
                "TOP FIVE TIME!  {:.1}s\n\nTYPE YOUR NAME\n> {}_\n\nLETTERS / NUMBERS / SPACE\nBACKSPACE  DELETE     ENTER  SAVE",
                stats.elapsed_secs, state.name_buffer
            )
        } else if stats.extracted {
            format!(
                "RUN COMPLETE  {:.1}s\n\nTOP 5 THIS SESSION\n\n{}\n\nPRESS R FOR NEXT PLAYER",
                stats.elapsed_secs,
                session.formatted()
            )
        } else {
            format!("TOP 5 THIS SESSION\n\n{}", session.formatted())
        };
    }
}
