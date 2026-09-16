use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::time::Virtual;

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
            .add_systems(Update, (guide_controls, update_guide_ui).chain())
            // Re-assert the pause after normal Update systems. This keeps the
            // survival/body simulation frozen while somebody is reading even
            // though the ordinary pause system may unpause virtual time.
            .add_systems(PostUpdate, enforce_guide_pause);
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
    // Permanent discoverability without filling the HUD with tutorial text.
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
                                    "1 // MOVE\nA / D or arrows move. SPACE jumps. Watch your landings: hard falls cause wounds, bleeding and fractures.\n\n\
2 // LOOT + USE\nWalk over supplies to collect them. Press 1-9 to select a hotbar slot, then F to use it. Food restores hunger; water restores thirst; medical gear keeps injuries from becoming fatal.\n\n\
3 // SURVIVE\nHunger slows you and can eventually damage health. Thirst becomes dangerous faster. If BOTH are low they make each other drain faster and health collapses faster too. Do not wait for 0%.\n\n\
4 // CRAFT\nPress C to open crafting. Craft bandages, splints and medkits from scavenged materials before you desperately need them.\n\n\
5 // ENEMIES\nPress E to attack nearby threats. Crawlers pressure you steadily; Skitters are faster and more aggressive deeper down. Avoid getting cornered when already injured or starving.\n\n\
6 // OBJECTIVE\nDescend through the cave, recover the LOST CARGO, then reach extraction. The mission panel at top-left always shows which half of the objective you are on.\n\n\
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
    mut guide: ResMut<GuideState>,
    mut leaderboard: ResMut<LeaderboardState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut velocity: Query<&mut LinearVelocity, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyG) {
        return;
    }

    guide.open = !guide.open;
    if guide.open {
        leaderboard.open = false;
        crafting.open = false;
        virtual_time.pause();
        if let Ok(mut velocity) = velocity.single_mut() {
            *velocity = LinearVelocity::ZERO;
        }
    } else {
        virtual_time.unpause();
    }
}

fn update_guide_ui(
    guide: Res<GuideState>,
    leaderboard: Res<LeaderboardState>,
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

    let dead = player
        .single()
        .map(|body| body.0.is_dead())
        .unwrap_or(false);
    if let Ok(mut prompt) = prompt.single_mut() {
        prompt.display = if guide.open || leaderboard.open || dead || stats.extracted {
            Display::None
        } else {
            Display::Flex
        };
    }
}

fn enforce_guide_pause(
    guide: Res<GuideState>,
    mut leaderboard: ResMut<LeaderboardState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut velocity: Query<&mut LinearVelocity, With<Player>>,
) {
    if !guide.open {
        return;
    }

    leaderboard.open = false;
    crafting.open = false;
    virtual_time.pause();
    if let Ok(mut velocity) = velocity.single_mut() {
        *velocity = LinearVelocity::ZERO;
    }
}
