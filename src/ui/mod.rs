//! HUD: vitals as bars, injuries, inventory, depth, and a full-screen
//! death overlay that forces a restart.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::items::{LastPickup, PlayerInventory};
use crate::physics::{Grounded, LandingImpact};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{CurrentDepth, RunStats};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLanding>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (capture_landings, update_hud, update_vital_bars, update_death_overlay));
    }
}

#[derive(Resource, Default)]
struct LastLanding {
    text: String,
}

#[derive(Component)]
struct HudText;

#[derive(Component, Clone, Copy)]
enum VitalKind {
    Blood,
    Hunger,
    Thirst,
    Stamina,
}

#[derive(Component)]
struct VitalFill(VitalKind);

#[derive(Component)]
struct DeathOverlay;

#[derive(Component)]
struct DeathOverlayText;

const BAR_LEFT: f32 = 92.0;
const BAR_WIDTH: f32 = 150.0;
const BAR_HEIGHT: f32 = 13.0;
const BAR_ROW_GAP: f32 = 20.0;
const BAR_TOP_START: f32 = 128.0;
const BAR_BG_COLOR: Color = Color::srgb(0.10, 0.09, 0.09);

fn spawn_hud(mut commands: Commands) {
    // top info block: depth, controls, movement state, injuries,
    // inventory, last landing / last item action.
    commands.spawn((
        HudText,
        Text::new(""),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.90, 0.86, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));

    let vitals = [
        (VitalKind::Blood, "blood", Color::srgb(0.78, 0.16, 0.16)),
        (VitalKind::Hunger, "hunger", Color::srgb(0.80, 0.58, 0.24)),
        (VitalKind::Thirst, "thirst", Color::srgb(0.28, 0.55, 0.85)),
        (VitalKind::Stamina, "stamina", Color::srgb(0.32, 0.72, 0.36)),
    ];

    for (i, (kind, label, color)) in vitals.into_iter().enumerate() {
        let row_top = BAR_TOP_START + i as f32 * BAR_ROW_GAP;

        commands.spawn((
            Text::new(label),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.78, 0.74, 0.68)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(row_top),
                left: Val::Px(16.0),
                ..default()
            },
        ));

        // bar background — fixed width, sits under the fill.
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(row_top),
                left: Val::Px(BAR_LEFT),
                width: Val::Px(BAR_WIDTH),
                height: Val::Px(BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
        ));

        // bar fill — same rect, width shrinks with the stat each frame.
        commands.spawn((
            VitalFill(kind),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(row_top),
                left: Val::Px(BAR_LEFT),
                width: Val::Px(BAR_WIDTH),
                height: Val::Px(BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(color),
        ));
    }

    // full-screen death banner, hidden until the player dies.
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
            BackgroundColor(Color::srgba(0.02, 0.01, 0.01, 0.93)),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("YOU DIED"),
                TextFont { font_size: 52.0, ..default() },
                TextColor(Color::srgb(0.82, 0.16, 0.16)),
            ));
            overlay.spawn((
                DeathOverlayText,
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.88, 0.84, 0.78)),
            ));
        });
}

fn capture_landings(mut events: MessageReader<LandingImpact>, mut last: ResMut<LastLanding>) {
    for impact in events.read() {
        last.text = format!(
            "hard landing  {:.0} u/s  severity {:.2}",
            impact.downward_speed, impact.severity
        );
    }
}

fn update_hud(
    player: Query<(&LinearVelocity, Option<&Grounded>, &Body, &PlayerInventory), With<Player>>,
    last: Res<LastLanding>,
    last_pickup: Res<LastPickup>,
    depth: Res<CurrentDepth>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok((velocity, grounded, body, inventory)) = player.single() else {
        return;
    };
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let grounded = if grounded.is_some() { "grounded" } else { "airborne" };
    let body_line = if body.0.wound_count() == 0 {
        "no injuries".to_string()
    } else {
        let status = if body.0.is_ko() { "   —  UNCONSCIOUS, press R" } else { "" };
        format!(
            "{} wound(s)   pain {:.1}   bleeding {:.2}/s{status}",
            body.0.wound_count(),
            body.0.total_pain(),
            body.0.total_bleed_rate()
        )
    };
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
    let inventory_line = format!(
        "inventory ({}/{}): {items}",
        inventory.0.used_weight(),
        inventory.0.capacity()
    );
    **text = format!(
        "DESCENT: NULL  —  depth layer {}\n\
         WASD/arrows move   Space jump   F use supplies   C craft   R restart run\n\
         {grounded}   vel ({:.0}, {:.0})\n\
         {body_line}\n\
         {inventory_line}\n\
         {}\n\
         {}",
        depth.0,
        velocity.x,
        velocity.y,
        last.text,
        last_pickup.text,
    );
}

fn update_vital_bars(
    player: Query<(&Body, &Survival), With<Player>>,
    mut fills: Query<(&VitalFill, &mut Node)>,
) {
    let Ok((body, survival)) = player.single() else {
        return;
    };
    for (fill, mut node) in &mut fills {
        let fraction = match fill.0 {
            VitalKind::Blood => body.0.blood_volume(),
            VitalKind::Hunger => survival.0.hunger(),
            VitalKind::Thirst => survival.0.thirst(),
            VitalKind::Stamina => survival.0.stamina(),
        };
        node.width = Val::Px(BAR_WIDTH * fraction.clamp(0.0, 1.0));
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
    if body.0.is_dead() {
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
