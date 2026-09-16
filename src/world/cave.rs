//! Procedural cave built around a guaranteed downward route.
//! Randomness adds side loot, hazards and enemies; it never gets to remove
//! the main descent. Progression is intentionally fall/ledge driven.

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
const X_BOUND: f32 = 760.0;
const FLOOR_Y: f32 = -(LAYER_HEIGHT * LAYER_COUNT as f32) - 200.0;
const MAIN_STEPS: usize = 24;
const CARGO_RADIUS: f32 = 58.0;
const EXTRACTION_RADIUS: f32 = 90.0;
const NORMAL_ACCELERATION: f32 = 1250.0;
const SURGE_ACCELERATION: f32 = 1850.0;
const SURGE_SECONDS: f32 = 8.0;
const ANNOUNCEMENT_SECONDS: f32 = 2.5;

#[derive(Resource, Default)] pub struct CurrentDepth(pub usize);
#[derive(Resource, Default)]
pub struct RunStats {
    pub deepest_layer: usize,
    pub elapsed_secs: f32,
    pub cargo_recovered: bool,
    pub extracted: bool,
}
#[derive(Resource, Default)]
pub struct DepthAnnouncement { pub text: String, pub remaining: f32 }
#[derive(Resource, Default)] struct HazardCooldown(f32);
#[derive(Resource, Default)] struct SurgeState { remaining: f32 }

#[derive(Component)] struct Cargo;
#[derive(Component)] struct ExtractionMarker;
#[derive(Component)] struct Hazard;
#[derive(Component)] struct SurgePickup;
#[derive(Component)] struct CaveObject;

pub struct CavePlugin;
impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentDepth>()
            .init_resource::<RunStats>()
            .init_resource::<DepthAnnouncement>()
            .init_resource::<HazardCooldown>()
            .init_resource::<SurgeState>()
            .add_systems(Startup, spawn_cave)
            .add_systems(Update, (track_depth, check_objective, damage_from_hazards, collect_surge, tick_surge, regenerate_on_restart).chain());
    }
}

fn spawn_cave(mut commands: Commands, asset_server: Res<AssetServer>) { generate_cave(&mut commands, &asset_server); }
fn extraction_pos() -> Vec2 { Vec2::new(-520.0, FLOOR_Y + 78.0) }
fn cargo_pos() -> Vec2 { Vec2::new(500.0, FLOOR_Y + 78.0) }

fn regenerate_on_restart(
    keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands, asset_server: Res<AssetServer>,
    existing: Query<Entity, With<CaveObject>>, mut depth: ResMut<CurrentDepth>,
    mut hazard_cooldown: ResMut<HazardCooldown>, mut surge: ResMut<SurgeState>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) { return; }
    for entity in &existing { commands.entity(entity).despawn(); }
    depth.0 = 0;
    hazard_cooldown.0 = 0.0;
    surge.remaining = 0.0;
    generate_cave(&mut commands, &asset_server);
}

fn generate_cave(commands: &mut Commands, asset_server: &AssetServer) {
    let mut rng = rand::thread_rng();
    let mut x = -420.0;
    let start_y = -40.0;
    spawn_platform(commands, asset_server, Vec2::new(x, start_y), Vec2::new(320.0, 30.0), 0);
    commands.spawn((
        CaveObject, Text2d::new("GO DOWN"), TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.90, 0.82, 0.56)), Transform::from_xyz(x + 70.0, start_y + 58.0, 2.0),
    ));

    let target_y = FLOOR_Y + 170.0;
    let base_drop = (start_y - target_y) / MAIN_STEPS as f32;
    let mut y = start_y;

    for step in 1..=MAIN_STEPS {
        let progress = step as f32 / MAIN_STEPS as f32;
        let layer = ((progress * LAYER_COUNT as f32).floor() as usize).min(LAYER_COUNT as usize - 1);
        let beginner = step <= 4;
        let drop = if beginner { base_drop * 0.82 } else { base_drop * rng.gen_range(0.88..1.12) };
        y -= drop;
        x = (x + rng.gen_range(-165.0..165.0)).clamp(-X_BOUND, X_BOUND);
        let width = if beginner { 300.0 } else { rng.gen_range(225.0..300.0) };
        let pos = Vec2::new(x, y);
        spawn_platform(commands, asset_server, pos, Vec2::new(width, 26.0), layer);

        if step == 2 {
            spawn_pickup(commands, asset_server, pos + Vec2::new(0.0, 31.0), ItemStack::new(ItemKind::Water, 1));
        } else if step == 4 {
            spawn_pickup(commands, asset_server, pos + Vec2::new(0.0, 31.0), ItemStack::new(ItemKind::Bandage, 1));
        } else if rng.gen_bool(0.58) {
            spawn_pickup(commands, asset_server, pos + Vec2::new(0.0, 31.0), random_item_for_layer(&mut rng, layer));
        }

        if layer > 0 && rng.gen_bool((0.18 + layer as f64 * 0.06).min(0.40)) {
            let enemy = spawn_enemy(commands, asset_server, pos + Vec2::new(rng.gen_range(-70.0..70.0), 24.0));
            commands.entity(enemy).insert(CaveObject);
        }
        if layer > 0 && rng.gen_bool(0.14) { spawn_hazard(commands, asset_server, pos + Vec2::new(rng.gen_range(-55.0..55.0), 20.0)); }
        if layer > 1 && rng.gen_bool(0.08) { spawn_surge(commands, asset_server, pos + Vec2::new(0.0, 45.0)); }

        if !beginner && rng.gen_bool(0.42) {
            let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            let ledge_x = (x + side * rng.gen_range(260.0..360.0)).clamp(-X_BOUND, X_BOUND);
            let ledge_pos = Vec2::new(ledge_x, y + rng.gen_range(-20.0..35.0));
            spawn_platform(commands, asset_server, ledge_pos, Vec2::new(130.0, 22.0), layer);
            spawn_pickup(commands, asset_server, ledge_pos + Vec2::new(0.0, 28.0), random_item_for_layer(&mut rng, layer));
            if layer > 1 && rng.gen_bool(0.35) {
                let enemy = spawn_enemy(commands, asset_server, ledge_pos + Vec2::new(0.0, 22.0));
                commands.entity(enemy).insert(CaveObject);
            }
        }
    }

    spawn_platform(commands, asset_server, Vec2::new(0.0, FLOOR_Y), Vec2::new(1800.0, 80.0), LAYER_COUNT as usize);

    let cargo = cargo_pos();
    commands.spawn((
        Cargo, CaveObject,
        Sprite { image: asset_server.load("sprites/item_cargo.png"), custom_size: Some(Vec2::splat(54.0)), ..default() },
        Transform::from_xyz(cargo.x, cargo.y, 1.0),
    ));
    commands.spawn((CaveObject, Text2d::new("MISSION CARGO"), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.96, 0.80, 0.45)), Transform::from_xyz(cargo.x, cargo.y + 45.0, 2.0)));

    let extract = extraction_pos();
    commands.spawn((
        ExtractionMarker, CaveObject,
        Sprite { color: Color::srgb(0.20, 0.68, 0.36), custom_size: Some(Vec2::new(110.0, 16.0)), ..default() },
        Transform::from_xyz(extract.x, FLOOR_Y + 45.0, 0.8),
    ));
    commands.spawn((CaveObject, Text2d::new("EXTRACTION"), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.55, 0.94, 0.60)), Transform::from_xyz(extract.x, extract.y + 40.0, 2.0)));

    for guard_x in [-250.0, 30.0, 270.0] {
        let enemy = spawn_enemy(commands, asset_server, Vec2::new(guard_x, FLOOR_Y + 64.0));
        commands.entity(enemy).insert(CaveObject);
    }
    spawn_hazard(commands, asset_server, Vec2::new(-80.0, FLOOR_Y + 46.0));
}

fn track_depth(
    time: Res<Time>, player: Query<&Transform, With<Player>>, mut depth: ResMut<CurrentDepth>,
    mut stats: ResMut<RunStats>, mut announcement: ResMut<DepthAnnouncement>,
) {
    let Ok(transform) = player.single() else { return; };
    let layer = (-transform.translation.y / LAYER_HEIGHT).floor().max(0.0) as usize;
    let clamped = layer.min(LAYER_COUNT as usize);
    if clamped != depth.0 {
        announcement.text = if clamped == 0 { "SURFACE".to_string() } else { format!("LAYER {clamped}") };
        announcement.remaining = ANNOUNCEMENT_SECONDS;
    }
    depth.0 = clamped;
    stats.deepest_layer = stats.deepest_layer.max(depth.0);
    if !stats.extracted { stats.elapsed_secs += time.delta_secs(); }
    if announcement.remaining > 0.0 { announcement.remaining = (announcement.remaining - time.delta_secs()).max(0.0); }
}

fn check_objective(
    mut commands: Commands, player: Query<&Transform, With<Player>>, cargo: Query<(Entity, &Transform), With<Cargo>>,
    mut stats: ResMut<RunStats>, mut last: ResMut<LastEvent>,
) {
    if stats.extracted { return; }
    let Ok(player) = player.single() else { return; };
    let player_pos = player.translation.truncate();
    if !stats.cargo_recovered {
        if let Ok((entity, cargo_transform)) = cargo.single() {
            if player_pos.distance(cargo_transform.translation.truncate()) <= CARGO_RADIUS {
                stats.cargo_recovered = true;
                commands.entity(entity).despawn();
                last.show("CARGO SECURED");
            }
        }
    } else if player_pos.distance(extraction_pos()) <= EXTRACTION_RADIUS {
        stats.extracted = true;
        last.show("MISSION COMPLETE");
    }
}

fn spawn_hazard(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    commands.spawn((
        Hazard, CaveObject,
        Sprite { image: asset_server.load("sprites/tile_rock_4.png"), color: Color::srgb(0.88, 0.16, 0.12), custom_size: Some(Vec2::new(72.0, 14.0)), ..default() },
        Transform::from_xyz(pos.x, pos.y, 0.7),
    )).with_children(|hazard| {
        hazard.spawn((
            Text2d::new("SPIKES"), TextFont { font_size: 9.0, ..default() }, TextColor(Color::srgb(1.0, 0.42, 0.34)),
            Transform::from_xyz(0.0, 16.0, 0.2),
        ));
    });
}

fn damage_from_hazards(
    time: Res<Time>, hazards: Query<&Transform, With<Hazard>>,
    mut player: Query<(&Transform, &mut Body), (With<Player>, Without<Hazard>)>,
    mut cooldown: ResMut<HazardCooldown>, mut last: ResMut<LastEvent>,
) {
    cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);
    if cooldown.0 > 0.0 { return; }
    let Ok((player_transform, mut body)) = player.single_mut() else { return; };
    let player_pos = player_transform.translation.truncate();
    for hazard in &hazards {
        let delta = player_pos - hazard.translation.truncate();
        if delta.x.abs() < 46.0 && delta.y.abs() < 32.0 {
            if let Some(wound) = landing_wound(BodyRegion::LeftLeg, 0.58) { body.0.apply_wound(wound); }
            cooldown.0 = 1.4;
            last.show("TRAP: SPIKES - BLEEDING");
            break;
        }
    }
}

fn spawn_surge(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    commands.spawn((
        SurgePickup, CaveObject,
        Sprite { image: asset_server.load("sprites/item_battery.png"), color: Color::srgb(0.45, 0.95, 1.0), custom_size: Some(Vec2::splat(30.0)), ..default() },
        Transform::from_xyz(pos.x, pos.y, 0.9),
    )).with_children(|pickup| {
        pickup.spawn((Text2d::new("SURGE"), TextFont { font_size: 10.0, ..default() }, TextColor(Color::srgb(0.55, 0.94, 1.0)), Transform::from_xyz(0.0, 25.0, 0.2)));
    });
}

fn collect_surge(
    mut commands: Commands, player: Query<&Transform, With<Player>>, pickups: Query<(Entity, &Transform), With<SurgePickup>>,
    mut surge: ResMut<SurgeState>, mut last: ResMut<LastEvent>,
) {
    let Ok(player) = player.single() else { return; };
    let player_pos = player.translation.truncate();
    for (entity, transform) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= 42.0 {
            commands.entity(entity).despawn();
            surge.remaining = SURGE_SECONDS;
            last.show("SURGE BOOST  8s");
            break;
        }
    }
}

fn tick_surge(time: Res<Time>, mut surge: ResMut<SurgeState>, mut player: Query<&mut MovementAcceleration, With<Player>>) {
    let Ok(mut acceleration) = player.single_mut() else { return; };
    if surge.remaining > 0.0 {
        surge.remaining = (surge.remaining - time.delta_secs()).max(0.0);
        acceleration.0 = SURGE_ACCELERATION;
    } else { acceleration.0 = NORMAL_ACCELERATION; }
}

fn tile_path(layer: usize) -> String { format!("sprites/tile_rock_{}.png", layer.min(4)) }

fn platform_edge_color(layer: usize) -> Color {
    match layer.min(4) {
        0 => Color::srgb(0.52, 0.49, 0.42),
        1 => Color::srgb(0.48, 0.44, 0.38),
        2 => Color::srgb(0.43, 0.39, 0.35),
        3 => Color::srgb(0.38, 0.35, 0.34),
        _ => Color::srgb(0.34, 0.32, 0.33),
    }
}

fn spawn_platform(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, size: Vec2, layer: usize) {
    commands.spawn((
        CaveObject,
        Sprite { image: asset_server.load(tile_path(layer)), custom_size: Some(size), ..default() },
        Transform::from_xyz(pos.x, pos.y, 0.0), RigidBody::Static, Collider::rectangle(size.x, size.y), Friction::new(0.85),
    )).with_children(|platform| {
        // A clean bright lip makes the safe landing surface obvious at a glance.
        platform.spawn((
            Sprite::from_color(platform_edge_color(layer), Vec2::new(size.x, 4.0)),
            Transform::from_xyz(0.0, size.y * 0.5 - 2.0, 0.2),
        ));
    });
}

fn spawn_pickup(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, item: ItemStack) {
    commands.spawn((
        CaveObject,
        Sprite { image: asset_server.load(item.kind.sprite_path()), custom_size: Some(Vec2::splat(24.0)), ..default() },
        Transform::from_xyz(pos.x, pos.y, 0.5), Pickup(item),
    )).with_children(|pickup| {
        pickup.spawn((
            Text2d::new(item.kind.label().to_uppercase()), TextFont { font_size: 10.0, ..default() },
            TextColor(Color::srgb(0.88, 0.86, 0.78)), Transform::from_xyz(0.0, 22.0, 0.2),
        ));
    });
}

fn random_item_for_layer(rng: &mut impl Rng, layer: usize) -> ItemStack {
    let pool: &[(ItemKind, u32)] = match layer {
        0 => &[(ItemKind::Scrap, 5), (ItemKind::Food, 4), (ItemKind::Water, 4), (ItemKind::Cloth, 3)],
        1 => &[(ItemKind::Scrap, 4), (ItemKind::Metal, 3), (ItemKind::Food, 3), (ItemKind::Water, 3), (ItemKind::Battery, 2), (ItemKind::Bandage, 2), (ItemKind::Splint, 1)],
        2 => &[(ItemKind::Metal, 4), (ItemKind::Battery, 3), (ItemKind::Bandage, 3), (ItemKind::Splint, 2), (ItemKind::Medkit, 1), (ItemKind::Water, 2)],
        _ => &[(ItemKind::Medkit, 3), (ItemKind::Battery, 3), (ItemKind::Metal, 3), (ItemKind::Bandage, 2), (ItemKind::Splint, 2), (ItemKind::Water, 2)],
    };
    let total: u32 = pool.iter().map(|(_, weight)| weight).sum();
    let mut roll = rng.gen_range(0..total);
    let mut kind = pool[0].0;
    for (candidate, weight) in pool {
        if roll < *weight { kind = *candidate; break; }
        roll -= weight;
    }
    ItemStack::new(kind, rng.gen_range(1..=2))
}
