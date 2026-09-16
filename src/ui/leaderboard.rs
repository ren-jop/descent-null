use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::Body;
use crate::player::Player;
use crate::world::{BestTimes, RunStats, SessionTimes};

#[derive(Resource, Default)]
pub struct LeaderboardState {
    pub open: bool,
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
            .add_systems(Startup, spawn_leaderboard)
            .add_systems(
                Update,
                (
                    leaderboard_controls,
                    update_mini_leaderboard,
                    update_full_leaderboard,
                ),
            )
            .add_systems(PostUpdate, pause_for_leaderboard);
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

fn best_time(times: &[f32]) -> String {
    times
        .first()
        .map(|seconds| format!("{seconds:.1}s"))
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
                width: Val::Px(190.0),
                min_height: Val::Px(82.0),
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
                        min_height: Val::Px(76.0),
                        padding: UiRect::all(Val::Px(9.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(panel_bg()),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        Text::new("RUN TIMES  [L]"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(pixel_highlight()),
                    ));
                    inner.spawn((
                        MiniLeaderboardText,
                        Text::new("SESSION  --\nPERSONAL --"),
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.86)),
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        width: Val::Px(650.0),
                        min_height: Val::Px(430.0),
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
                                min_height: Val::Px(420.0),
                                padding: UiRect::all(Val::Px(28.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(18.0),
                                ..default()
                            },
                            BackgroundColor(panel_bg()),
                        ))
                        .with_children(|inner| {
                            inner.spawn((
                                Text::new("DESCENT RECORDS"),
                                TextFont {
                                    font_size: 30.0,
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
                                Text::new("THIS SESSION\nNo completed runs yet\n\nPERSONAL BESTS\nNo completed runs yet"),
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

fn leaderboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LeaderboardState>,
    mini_interaction: Query<&Interaction, (Changed<Interaction>, With<MiniLeaderboardButton>)>,
    full_interaction: Query<&Interaction, (Changed<Interaction>, With<FullLeaderboardButton>)>,
) {
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

fn pause_for_leaderboard(
    state: Res<LeaderboardState>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if state.open {
        virtual_time.pause();
    }
}

fn update_mini_leaderboard(
    best: Res<BestTimes>,
    session: Res<SessionTimes>,
    state: Res<LeaderboardState>,
    player: Query<&Body, With<Player>>,
    stats: Res<RunStats>,
    mut frame: Query<&mut Node, With<MiniLeaderboard>>,
    mut text: Query<&mut Text, With<MiniLeaderboardText>>,
) {
    let dead = player
        .single()
        .map(|body| body.0.is_dead())
        .unwrap_or(false);
    if let Ok(mut frame) = frame.single_mut() {
        frame.display = if state.open || dead || stats.extracted {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut text) = text.single_mut() {
        **text = format!(
            "SESSION  {}\nPERSONAL {}",
            best_time(&session.times),
            best_time(&best.times)
        );
    }
}

fn update_full_leaderboard(
    best: Res<BestTimes>,
    session: Res<SessionTimes>,
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
    if state.open {
        if let Ok(mut text) = text.single_mut() {
            **text = format!(
                "THIS SESSION\n{}\n\nPERSONAL BESTS\n{}",
                session.formatted(),
                best.formatted()
            );
        }
    }
}
