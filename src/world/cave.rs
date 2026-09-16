//! procedural cave: several depth layers of platforms with randomized
//! gaps, occasional side ledges to explore, and scattered item pickups.
//! regenerates differently every run — both on first launch and again
//! every time the player restarts with `R` (see regenerate_on_restart).

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::enemy::spawn_enemy;
use crate::items::{ItemKind, ItemStack, Pickup};
use crate::player::Player;

const LAYER_COUNT: i32 = 4;
const LAYER_HEIGHT: f32 = 700.0;
const PLATFORMS_PER_LAYER: std::ops::Range<i32> = 5..8;
const X_BOUND: f32 = 820.0;

#[derive(Resource, Default)]
pub struct CurrentDepth(pub usize);

/// stats for the death/win screens — deepest layer reached, time survived,
/// and whether the objective was recovered. reset in player::reset_player
/// alongside everything else.
#[derive(Resource, Default)]
pub struct RunStats {
    pub deepest_layer: usize,
    pub elapsed_secs: f32,
    pub extracted: bool,
}

/// brief "Layer N" banner shown when the player crosses into a new depth
/// layer. ui::spawn_hud renders it; this module just drives the timer.
#[derive(Resource, Default)]
pub struct DepthAnnouncement {
    pub text: String,
    pub remaining: f32,
}

const ANNOUNCEMENT_SECONDS: f32 = 2.5;
const EXTRACTION_RADIUS: f32 = 54.0;

#[derive(Component)]
struct Cargo;

/// tags every entity the cave generator creates (platforms, pickups,
/// cargo, enemies) so a restart can despawn the whole layout and build
/// a fresh one, instead of resetting the player into the same cave.
#[derive(Component)]
struct CaveObject;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentDepth>()
            .init_resource::<RunStats>()
            .init_resource::<DepthAnnouncement>()
            .add_systems(Startup, spawn_cave)
            .add_systems(Update, (track_depth, check_extraction, regenerate_on_restart));
    }
}

fn spawn_cave(mut commands: Commands, asset_server: Res<AssetServer>) {
    generate_cave(&mut commands, &asset_server);
}

/// `R` — on top of the player's own reset (position/body/vitals/
/// inventory, handled in player::reset_player), the cave itself
/// despawns and regenerates so every run is a genuinely different
/// layout, not the same one replayed.
fn regenerate_on_restart(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<CaveObject>>,
    mut depth: ResMut<CurrentDepth>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    depth.0 = 0;
    generate_cave(&mut commands, &asset_server);
}

fn generate_cave(commands: &mut Commands, asset_server: &AssetServer) {
    let mut rng = rand::thread_rng();

    // starting ledge — fixed, so the opening always plays the same.
    let mut x = -420.0;
    let mut y = -40.0;
    spawn_platform(commands, asset_server, Vec2::new(x, y), Vec2::new(280.0, 28.0), 0);

    // wide catch floor far below everything, so a bad run ends in a
    // landing instead of an infinite fall.
    let floor_y = -(LAYER_HEIGHT * LAYER_COUNT as f32) - 200.0;
    spawn_platform(commands, asset_server, Vec2::new(0.0, floor_y), Vec2::new(3200.0, 80.0), LAYER_COUNT as usize);
    commands.spawn((
        Cargo,
        CaveObject,
        Sprite {
            image: asset_server.load("sprites/item_cargo.png"),
            // The run objective should be immediately distinguishable from
            // ordinary loot when the player finally reaches the bottom.
            custom_size: Some(Vec2::splat(46.0)),
            ..default()
        },
        Transform::from_xyz(300.0, floor_y + 40.0 + 23.0, 0.5),
    ));

    for layer in 0..LAYER_COUNT {
        let layer_top = -(layer as f32) * LAYER_HEIGHT;
        let platform_count = rng.gen_range(PLATFORMS_PER_LAYER);

        for _ in 0..platform_count {
            let width = rng.gen_range(140.0..260.0);
            x = (x + rng.gen_range(-260.0..260.0)).clamp(-X_BOUND, X_BOUND);
            y -= rng.gen_range(90.0..170.0);
            // clamp keeps this platform inside its own layer's band.
            y = y.max(layer_top - LAYER_HEIGHT + 60.0);

            spawn_platform(commands, asset_server, Vec2::new(x, y), Vec2::new(width, 24.0), layer as usize);

            if rng.gen_bool(0.7) {
                let item = random_item_for_layer(&mut rng, layer as usize);
                spawn_pickup(commands, asset_server, Vec2::new(x, y + 28.0), item);
            }

            // enemies start appearing from layer 1 — the opening layer
            // stays safe so a new run always has a moment to get its
            // bearings before anything can hurt it.
            if layer > 0 && rng.gen_bool(0.22) {
                let enemy = spawn_enemy(commands, asset_server, Vec2::new(x, y + 20.0));
                commands.entity(enemy).insert(CaveObject);
            }

            // occasional side ledge — a short dead-end off to one side,
            // worth a detour rather than sitting on the main path down.
            if rng.gen_bool(0.35) {
                let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                let ledge_x = (x + side * rng.gen_range(260.0..380.0)).clamp(-X_BOUND, X_BOUND);
                let ledge_pos = Vec2::new(ledge_x, y + rng.gen_range(-30.0..40.0));
                spawn_platform(commands, asset_server, ledge_pos, Vec2::new(110.0, 20.0), layer as usize);
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
    stats.elapsed_secs += time.delta_secs();
    if announcement.remaining > 0.0 {
        announcement.remaining = (announcement.remaining - time.delta_secs()).max(0.0);
    }
}

fn check_extraction(
    player: Query<&Transform, With<Player>>,
    cargo: Query<&Transform, With<Cargo>>,
    mut stats: ResMut<RunStats>,
) {
    if stats.extracted {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(cargo) = cargo.single() else {
        return;
    };
    if player.translation.truncate().distance(cargo.translation.truncate()) <= EXTRACTION_RADIUS {
        stats.extracted = true;
    }
}

// tile textures only exist for layers 0..=4 (4 is also used for the
// bottom catch floor) — clamp so a deeper layer count wouldn't panic.
fn tile_path(layer: usize) -> String {
    format!("sprites/tile_rock_{}.png", layer.min(4))
}

fn spawn_platform(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, size: Vec2, layer: usize) {
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

fn spawn_pickup(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, item: ItemStack) {
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

// weighted pool that shifts toward rarer/better items with depth.
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
        ],
    };
    let total: u32 = pool.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0..total);
    let mut kind = pool[0].0;
    for (candidate, weight) in pool {
        if roll < *weight {
            kind = *candidate;
            break;
        }
        roll -= weight;
    }
    ItemStack::new(kind, rng.gen_range(1..=3))
}
