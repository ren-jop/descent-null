//! Procedural cave with a readable mission loop:
//! descend -> survive hazards/enemies -> recover cargo -> climb back to EXIT.
//! The cave regenerates on every restart.

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::body::{landing_wound, Body, BodyRegion};
use crate::enemy::spawn_enemy;
use crate::items::{ItemKind, ItemStack, LastEvent, Pickup};
use crate::physics::MovementAcceleration;
use crate::player::Player;

const LAYER_COUNT: i32 = 4;
const LAYER_HEIGHT: f32 = 700.0;
const PLATFORMS_PER_LAYER: std::ops::Range<i32> = 6..9;
const X_BOUND: f32 = 820.0;
const CARGO_RADIUS: f32 = 58.0;
const EXIT_RADIUS: f32 = 95.0;
const EXIT_POS: Vec2 = Vec2::new(-420.0, 40.0);
const NORMAL_ACCELERATION: f32 = 1250.0;
const SURGE_ACCELERATION: f32 = 1850.0;
const SURGE_SECONDS: f32 = 8.0;

#[derive(Resource, Default)]
pub struct CurrentDepth(pub usize);

#[derive(Resource, Default)]
pub struct RunStats {
    pub deepest_layer: usize,
    pub elapsed_secs: f32,
    pub cargo_recovered: bool,
    pub extracted: bool,
}

#[derive(Resource, Default)]
pub struct DepthAnnouncement {
    pub text: String,
    pub remaining: f32,
}

#[derive(Resource, Default)]
struct HazardCooldown(f32);

#[derive(Resource, Default)]
struct SurgeState {
    remaining: f32,
}

const ANNOUNCEMENT_SECONDS: f32 = 2.5;

#[derive(Component)]
struct Cargo;
#[derive(Component)]
struct ExitMarker;
#[derive(Component)]
struct Hazard;
#[derive(Component)]
struct SurgePickup;
#[derive(Component)]
struct CaveObject;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentDepth>()
            .init_resource::<RunStats>()
            .init_resource::<DepthAnnouncement>()
            .init_resource::<HazardCooldown>()
            .init_resource::<SurgeState>()
            .add_systems(Startup, spawn_cave)
            .add_systems(
                Update,
                (
                    track_depth,
                    check_objective,
                    damage_from_hazards,
                    collect_surge,
                    tick_surge,
                    regenerate_on_restart,
                )
                    .chain(),
            );
    }
}

fn spawn_cave(mut commands: Commands, asset_server: Res<AssetServer>) {
    generate_cave(&mut commands, &asset_server);
}

fn regenerate_on_restart(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<CaveObject>>,
    mut depth: ResMut<CurrentDepth>,
    mut hazard_cooldown: ResMut<HazardCooldown>,
    mut surge: ResMut<SurgeState>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    depth.0 = 0;
    hazard_cooldown.0 = 0.0;
    surge.remaining = 0.0;
    generate_cave(&mut commands, &asset_server);
}

fn generate_cave(commands: &mut Commands, asset_server: &AssetServer) {
    let mut rng = rand::thread_rng();

    let mut x = -420.0;
    let mut y = -40.0;
    spawn_platform(
        commands,
        asset_server,
        Vec2::new(x, y),
        Vec2::new(300.0, 30.0),
        0,
    );

    // The start is also the extraction point after cargo recovery.
    commands.spawn((
        ExitMarker,
        CaveObject,
        Text2d::new("EXIT"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.55, 0.92, 0.58)),
        Transform::from_xyz(EXIT_POS.x, EXIT_POS.y + 48.0, 2.0),
    ));

    let floor_y = -(LAYER_HEIGHT * LAYER_COUNT as f32) - 200.0;
    spawn_platform(
        commands,
        asset_server,
        Vec2::new(0.0, floor_y),
        Vec2::new(3200.0, 80.0),
        LAYER_COUNT as usize,
    );

    commands.spawn((
        Cargo,
        CaveObject,
        Sprite {
            image: asset_server.load("sprites/item_cargo.png"),
            custom_size: Some(Vec2::splat(52.0)),
            ..default()
        },
        Transform::from_xyz(300.0, floor_y + 70.0, 1.0),
    ));

    // The objective chamber is guarded. Reaching the bottom is the middle
    // of the mission now, not an instant win.
    for guard_x in [150.0, 275.0, 420.0] {
        let enemy = spawn_enemy(commands, asset_server, Vec2::new(guard_x, floor_y + 65.0));
        commands.entity(enemy).insert(CaveObject);
    }

    for layer in 0..LAYER_COUNT {
        let layer_top = -(layer as f32) * LAYER_HEIGHT;
        let platform_count = rng.gen_range(PLATFORMS_PER_LAYER);

        for _ in 0..platform_count {
            let width = rng.gen_range(150.0..290.0);
            x = (x + rng.gen_range(-260.0..260.0)).clamp(-X_BOUND, X_BOUND);
            y -= rng.gen_range(85.0..155.0);
            y = y.max(layer_top - LAYER_HEIGHT + 60.0);

            spawn_platform(
                commands,
                asset_server,
                Vec2::new(x, y),
                Vec2::new(width, 26.0),
                layer as usize,
            );

            if rng.gen_bool(0.68) {
                let item = random_item_for_layer(&mut rng, layer as usize);
                spawn_pickup(commands, asset_server, Vec2::new(x, y + 30.0), item);
            }

            if layer > 0 {
                let enemy_chance = 0.24 + layer as f64 * 0.055;
                if rng.gen_bool(enemy_chance.min(0.46)) {
                    let enemy = spawn_enemy(commands, asset_server, Vec2::new(x, y + 22.0));
                    commands.entity(enemy).insert(CaveObject);
                }

                if rng.gen_bool(0.18) {
                    let hazard_x = x + rng.gen_range(-45.0..45.0);
                    spawn_hazard(commands, asset_server, Vec2::new(hazard_x, y + 20.0));
                }

                if rng.gen_bool(0.10) {
                    spawn_surge(commands, asset_server, Vec2::new(x, y + 42.0));
                }
            }

            if rng.gen_bool(0.38) {
                let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                let ledge_x =
                    (x + side * rng.gen_range(250.0..370.0)).clamp(-X_BOUND, X_BOUND);
                let ledge_pos = Vec2::new(ledge_x, y + rng.gen_range(-35.0..45.0));
                spawn_platform(
                    commands,
                    asset_server,
                    ledge_pos,
                    Vec2::new(120.0, 22.0),
                    layer as usize,
                );
                let item = random_item_for_layer(&mut rng, layer as usize);
                spawn_pickup(commands, asset_server, ledge_pos + Vec2::new(0.0, 28.0), item);
            }
        }
    }
}

fn track_depth(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut depth: ResMut<CurrentDepth>,
    mut stats: ResMut<RunStats>,
    mut announcement: ResMut<DepthAnnouncement>,
) {
    let Ok(transform) = player.single() else {
        return;
    };
    let layer = (-transform.translation.y / LAYER_HEIGHT).floor().max(0.0) as usize;
    let clamped = layer.min(LAYER_COUNT as usize);
    if clamped != depth.0 {
        announcement.text = format!("Layer {clamped}");
        announcement.remaining = ANNOUNCEMENT_SECONDS;
    }
    depth.0 = clamped;
    stats.deepest_layer = stats.deepest_layer.max(depth.0);
    if !stats.extracted {
        stats.elapsed_secs += time.delta_secs();
    }
    if announcement.remaining > 0.0 {
        announcement.remaining = (announcement.remaining - time.delta_secs()).max(0.0);
    }
}

fn check_objective(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    cargo: Query<(Entity, &Transform), With<Cargo>>,
    mut stats: ResMut<RunStats>,
    mut last: ResMut<LastEvent>,
) {
    if stats.extracted {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let player_pos = player.translation.truncate();

    if !stats.cargo_recovered {
        if let Ok((entity, cargo_transform)) = cargo.single() {
            if player_pos.distance(cargo_transform.translation.truncate()) <= CARGO_RADIUS {
                stats.cargo_recovered = true;
                commands.entity(entity).despawn();
                last.show("CARGO RECOVERED - return to the EXIT at the surface");
            }
        }
    } else if player_pos.distance(EXIT_POS) <= EXIT_RADIUS {
        stats.extracted = true;
        last.show("MISSION COMPLETE - cargo extracted");
    }
}

fn spawn_hazard(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    commands.spawn((
        Hazard,
        CaveObject,
        Sprite {
            image: asset_server.load("sprites/tile_rock_4.png"),
            color: Color::srgb(0.72, 0.22, 0.18),
            custom_size: Some(Vec2::new(72.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.7),
    ));
}

fn damage_from_hazards(
    time: Res<Time>,
    hazards: Query<&Transform, With<Hazard>>,
    mut player: Query<(&Transform, &mut Body), (With<Player>, Without<Hazard>)>,
    mut cooldown: ResMut<HazardCooldown>,
    mut last: ResMut<LastEvent>,
) {
    cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);
    if cooldown.0 > 0.0 {
        return;
    }
    let Ok((player_transform, mut body)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    for hazard in &hazards {
        let delta = player_pos - hazard.translation.truncate();
        if delta.x.abs() < 46.0 && delta.y.abs() < 32.0 {
            if let Some(wound) = landing_wound(BodyRegion::LeftLeg, 0.58) {
                body.0.apply_wound(wound);
            }
            cooldown.0 = 1.4;
            last.show("SPIKES - leg cut and bleeding");
            break;
        }
    }
}

fn spawn_surge(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    commands.spawn((
        SurgePickup,
        CaveObject,
        Sprite {
            image: asset_server.load("sprites/item_battery.png"),
            color: Color::srgb(0.45, 0.95, 1.0),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.9),
    ));
}

fn collect_surge(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    pickups: Query<(Entity, &Transform), With<SurgePickup>>,
    mut surge: ResMut<SurgeState>,
    mut last: ResMut<LastEvent>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let player_pos = player.translation.truncate();
    for (entity, transform) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= 42.0 {
            commands.entity(entity).despawn();
            surge.remaining = SURGE_SECONDS;
            last.show("NULL SURGE - movement boosted for 8 seconds");
            break;
        }
    }
}

fn tick_surge(
    time: Res<Time>,
    mut surge: ResMut<SurgeState>,
    mut player: Query<&mut MovementAcceleration, With<Player>>,
) {
    let Ok(mut acceleration) = player.single_mut() else {
        return;
    };
    if surge.remaining > 0.0 {
        surge.remaining = (surge.remaining - time.delta_secs()).max(0.0);
        acceleration.0 = SURGE_ACCELERATION;
    } else {
        acceleration.0 = NORMAL_ACCELERATION;
    }
}

fn tile_path(layer: usize) -> String {
    format!("sprites/tile_rock_{}.png", layer.min(4))
}

fn spawn_platform(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    size: Vec2,
    layer: usize,
) {
    commands.spawn((
        CaveObject,
        Sprite {
            image: asset_server.load(tile_path(layer)),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        Friction::new(0.85),
    ));
}

fn spawn_pickup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    item: ItemStack,
) {
    commands.spawn((
        CaveObject,
        Sprite {
            image: asset_server.load(item.kind.sprite_path()),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.5),
        Pickup(item),
    ));
}

fn random_item_for_layer(rng: &mut impl Rng, layer: usize) -> ItemStack {
    let pool: &[(ItemKind, u32)] = match layer {
        0 => &[
            (ItemKind::Scrap, 5),
            (ItemKind::Food, 4),
            (ItemKind::Water, 4),
            (ItemKind::Cloth, 3),
        ],
        1 => &[
            (ItemKind::Scrap, 4),
            (ItemKind::Metal, 3),
            (ItemKind::Food, 3),
            (ItemKind::Water, 3),
            (ItemKind::Battery, 2),
            (ItemKind::Bandage, 2),
            (ItemKind::Splint, 1),
        ],
        2 => &[
            (ItemKind::Metal, 4),
            (ItemKind::Battery, 3),
            (ItemKind::Bandage, 3),
            (ItemKind::Splint, 2),
            (ItemKind::Medkit, 1),
            (ItemKind::Water, 2),
        ],
        _ => &[
            (ItemKind::Medkit, 3),
            (ItemKind::Battery, 3),
            (ItemKind::Metal, 3),
            (ItemKind::Bandage, 2),
            (ItemKind::Splint, 2),
            (ItemKind::Water, 2),
        ],
    };
    let total: u32 = pool.iter().map(|(_, weight)| weight).sum();
    let mut roll = rng.gen_range(0..total);
    let mut kind = pool[0].0;
    for (candidate, weight) in pool {
        if roll < *weight {
            kind = *candidate;
            break;
        }
        roll -= weight;
    }
    ItemStack::new(kind, rng.gen_range(1..=2))
}
