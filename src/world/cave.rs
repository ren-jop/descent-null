//! procedural cave: several depth layers of platforms with randomized
//! gaps, occasional side ledges to explore, and scattered item pickups.
//! regenerates differently every run (see spawn_cave's rng seed).

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::items::{ItemKind, ItemStack, Pickup};
use crate::player::Player;

const LAYER_COUNT: i32 = 4;
const LAYER_HEIGHT: f32 = 700.0;
const PLATFORMS_PER_LAYER: std::ops::Range<i32> = 5..8;
const X_BOUND: f32 = 820.0;

#[derive(Resource, Default)]
pub struct CurrentDepth(pub usize);

/// stats for the death screen — deepest layer reached and time survived
/// this run. reset in player::reset_player alongside everything else.
#[derive(Resource, Default)]
pub struct RunStats {
    pub deepest_layer: usize,
    pub elapsed_secs: f32,
}

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentDepth>()
            .init_resource::<RunStats>()
            .add_systems(Startup, spawn_cave)
            .add_systems(Update, track_depth);
    }
}

fn spawn_cave(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();

    // starting ledge — fixed, so the opening always plays the same.
    let mut x = -420.0;
    let mut y = -40.0;
    spawn_platform(&mut commands, &asset_server, Vec2::new(x, y), Vec2::new(280.0, 28.0), 0);

    // wide catch floor far below everything, so a bad run ends in a
    // landing instead of an infinite fall.
    let floor_y = -(LAYER_HEIGHT * LAYER_COUNT as f32) - 200.0;
    spawn_platform(
        &mut commands,
        &asset_server,
        Vec2::new(0.0, floor_y),
        Vec2::new(3200.0, 80.0),
        LAYER_COUNT as usize,
    );

    for layer in 0..LAYER_COUNT {
        let layer_top = -(layer as f32) * LAYER_HEIGHT;
        let platform_count = rng.gen_range(PLATFORMS_PER_LAYER);

        for _ in 0..platform_count {
            let width = rng.gen_range(140.0..260.0);
            x = (x + rng.gen_range(-260.0..260.0)).clamp(-X_BOUND, X_BOUND);
            y -= rng.gen_range(90.0..170.0);
            // clamp keeps this platform inside its own layer's band.
            y = y.max(layer_top - LAYER_HEIGHT + 60.0);

            spawn_platform(&mut commands, &asset_server, Vec2::new(x, y), Vec2::new(width, 24.0), layer as usize);

            if rng.gen_bool(0.7) {
                let item = random_item_for_layer(&mut rng, layer as usize);
                spawn_pickup(&mut commands, &asset_server, Vec2::new(x, y + 24.0), item);
            }

            // occasional side ledge — a short dead-end off to one side,
            // worth a detour rather than sitting on the main path down.
            if rng.gen_bool(0.35) {
                let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                let ledge_x = (x + side * rng.gen_range(260.0..380.0)).clamp(-X_BOUND, X_BOUND);
                let ledge_pos = Vec2::new(ledge_x, y + rng.gen_range(-30.0..40.0));
                spawn_platform(&mut commands, &asset_server, ledge_pos, Vec2::new(110.0, 20.0), layer as usize);
                let item = random_item_for_layer(&mut rng, layer as usize);
                spawn_pickup(&mut commands, &asset_server, ledge_pos + Vec2::new(0.0, 24.0), item);
            }
        }
    }
}

fn track_depth(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut depth: ResMut<CurrentDepth>,
    mut stats: ResMut<RunStats>,
) {
    let Ok(transform) = player.single() else {
        return;
    };
    let layer = (-transform.translation.y / LAYER_HEIGHT).floor().max(0.0) as usize;
    depth.0 = layer.min(LAYER_COUNT as usize);
    stats.deepest_layer = stats.deepest_layer.max(depth.0);
    stats.elapsed_secs += time.delta_secs();
}

// tile textures only exist for layers 0..=4 (4 is also used for the
// bottom catch floor) — clamp so a deeper layer count wouldn't panic.
fn tile_path(layer: usize) -> String {
    format!("sprites/tile_rock_{}.png", layer.min(4))
}

fn spawn_platform(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, size: Vec2, layer: usize) {
    commands.spawn((
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
        Sprite {
            image: asset_server.load(item_sprite_path(item.kind)),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.5),
        Pickup(item),
    ));
}

fn item_sprite_path(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Scrap => "sprites/item_scrap.png",
        ItemKind::Cloth => "sprites/item_cloth.png",
        ItemKind::Metal => "sprites/item_metal.png",
        ItemKind::Food => "sprites/item_food.png",
        ItemKind::Water => "sprites/item_water.png",
        ItemKind::Battery => "sprites/item_battery.png",
        ItemKind::Bandage => "sprites/item_bandage.png",
        ItemKind::Splint => "sprites/item_splint.png",
        ItemKind::Medkit => "sprites/item_medkit.png",
    }
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
