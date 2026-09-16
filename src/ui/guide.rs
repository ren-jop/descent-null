use bevy::prelude::*;

use crate::body::Body;
use crate::items::CraftingMenu;
use crate::player::Player;
use crate::world::RunStats;

use super::leaderboard::LeaderboardState;

#[derive(Resource, Default)]
pub struct GuideState {
    pub open: bool,
}

#[derive(Component)]
struct GuideOverlay;
#[derive(Component)]
struct GuidePrompt;

pub struct GuidePlugin;

impl Plugin for GuidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuideState>()
            .add_systems(Startup, spawn_guide_ui)
            .add_systems(Update, (guide_controls, update_guide_ui).chain());
    }
}

fn panel_bg() -> Color {
    Color::srgba(0.025, 0.022, 0.020, 0.985)
}

fn pixel_border() -> Color {
    Color::srgb(0.43, 0.39, 0.29)
}

fn pixel_highlight() -> Color {
    Color::srgb(0.80, 0.67, 0.34)
}

fn spawn_guide_ui(mut commands: Commands) {
    commands.spawn((
        GuidePrompt,
        Text::new("[G]  WALKTHROUGH"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(pixel_highlight()),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            right: Val::Px(18.0),
            padding: UiRect::all(Val::Px(7.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.82)),
    ));

    commands
        .spawn((
            GuideOverlay,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.93)),
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        width: Val::Px(760.0),
                        max_width: Val::Percent(92.0),
                        min_height: Val::Px(560.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(pixel_border()),
                ))
                .with_children(|outer| {
                    outer
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(550.0),
                                padding: UiRect::all(Val::Px(28.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(14.0),
                                ..default()
                            },
                            BackgroundColor(panel_bg()),
                        ))
                        .with_children(|inner| {
                            inner.spawn((
                                Text::new("WALKTHROUGH // HOW TO SURVIVE THE DESCENT"),
                                TextFont {
                                    font_size: 25.0,
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
                                Text::new(
                                    "1 // MOVE\nA / D or arrows move. SPACE jumps. Hard falls cause wounds, bleeding and fractures.\n\n\
2 // LOOT + USE\nWalk over supplies to collect them. Press 1-9 to select a hotbar slot, then F to use it. Food restores hunger; water restores thirst.\n\n\
3 // SURVIVE\nLow hunger slows movement and can eventually hurt health. Low thirst narrows vision and becomes dangerous faster. When BOTH are low they drain each other faster and health falls faster too.\n\n\
4 // CRAFT\nPress C to open crafting. Make bandages, splints and medkits before you desperately need them.\n\n\
5 // ENEMIES\nPress E to attack nearby threats. Crawlers pressure you steadily. Silverfish are faster, detect you earlier and can poison you; poison removes health in visible one-second ticks.\n\n\
6 // OBJECTIVE\nDescend through the cave, recover the LOST CARGO, then reach extraction. The top-left mission panel always shows the current objective.\n\n\
EXPO // RECORDS\nPress L to view the current session top five. A qualifying finish lets you enter your name before the next player starts.\n\n\
[G] CLOSE WALKTHROUGH"
                                ),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.92, 0.90, 0.83)),
                            ));
                        });
                });
        });
}

fn guide_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    stats: Res<RunStats>,
    mut guide: ResMut<GuideState>,
    mut leaderboard: ResMut<LeaderboardState>,
    mut crafting: ResMut<CraftingMenu>,
) {
    if !keyboard.just_pressed(KeyCode::KeyG) {
        return;
    }

    // The result/name-entry screen owns input after extraction. This avoids G
    // stealing focus while an expo player is typing their leaderboard name.
    if stats.extracted || leaderboard.name_entry {
        return;
    }

    guide.open = !guide.open;
    if guide.open {
        leaderboard.open = false;
        crafting.open = false;
    }
}

fn update_guide_ui(
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
    crafting: Res<CraftingMenu>,
    stats: Res<RunStats>,
    player: Query<&Body, With<Player>>,
    mut overlay: Query<&mut Node, (With<GuideOverlay>, Without<GuidePrompt>)>,
    mut prompt: Query<&mut Node, (With<GuidePrompt>, Without<GuideOverlay>)>,
) {
    if let Ok(mut overlay) = overlay.single_mut() {
        overlay.display = if guide.open {
            Display::Flex
        } else {
            Display::None
        };
    }

    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    if let Ok(mut prompt) = prompt.single_mut() {
        prompt.display = if guide.open
            || leaderboard.open
            || crafting.open
            || dead
            || stats.extracted
        {
            Display::None
        } else {
            Display::Flex
        };
    }
}
