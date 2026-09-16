//! Player spawn, camera follow, landing shake and run reset.

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::body::Body;
use crate::items::{PlayerInventory, SelectedSlot};
use crate::physics::{CharacterControllerBundle, LandingImpact};
use crate::survival::Survival;
use crate::world::RunStats;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnHint>()
            .init_resource::<CameraFollow>()
            .init_resource::<CameraShake>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    follow_camera,
                    add_shake_on_landing,
                    apply_camera_shake,
                    follow_camera_with_vignette,
                    tick_spawn_hint,
                    reset_player,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FollowCamera;

#[derive(Component)]
struct Vignette;

const HINT_SECONDS: f32 = 7.0;

#[derive(Resource)]
pub struct SpawnHint(pub f32);

impl Default for SpawnHint {
    fn default() -> Self {
        Self(HINT_SECONDS)
    }
}

const SPAWN: Vec3 = Vec3::new(-420.0, 40.0, 0.0);
const CAMERA_Y_OFFSET: f32 = 58.0;

#[derive(Resource)]
struct CameraFollow {
    position: Vec3,
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            position: Vec3::new(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0),
        }
    }
}

#[derive(Resource, Default)]
struct CameraShake {
    trauma: f32,
}

const SHAKE_DECAY_PER_SEC: f32 = 1.4;
const MAX_SHAKE_OFFSET: f32 = 18.0;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Player,
            Transform::from_translation(SPAWN),
            Visibility::default(),
            CharacterControllerBundle::new(Collider::capsule(12.5, 20.0)),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            ColliderDensity(2.0),
            GravityScale(1.5),
            TransformInterpolation,
            Body::default(),
            Survival::default(),
            PlayerInventory::default(),
        ))
        .with_children(|player| {
            // Simple original explorer silhouette built from crisp shapes so
            // the character reads clearly at gameplay scale. This replaces
            // the distorted placeholder PNG until a full animated sprite
            // sheet is authored.
            player.spawn((
                Sprite::from_color(Color::srgb(0.19, 0.22, 0.24), Vec2::new(22.0, 25.0)),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            // backpack
            player.spawn((
                Sprite::from_color(Color::srgb(0.12, 0.14, 0.15), Vec2::new(7.0, 19.0)),
                Transform::from_xyz(-13.0, 0.0, 0.1),
            ));
            // head / face
            player.spawn((
                Sprite::from_color(Color::srgb(0.72, 0.58, 0.43), Vec2::new(15.0, 13.0)),
                Transform::from_xyz(0.0, 18.0, 0.2),
            ));
            // helmet
            player.spawn((
                Sprite::from_color(Color::srgb(0.70, 0.56, 0.22), Vec2::new(18.0, 7.0)),
                Transform::from_xyz(0.0, 25.0, 0.3),
            ));
            // helmet lamp
            player.spawn((
                Sprite::from_color(Color::srgb(0.96, 0.88, 0.54), Vec2::new(5.0, 5.0)),
                Transform::from_xyz(6.0, 26.0, 0.4),
            ));
            // legs
            player.spawn((
                Sprite::from_color(Color::srgb(0.11, 0.13, 0.14), Vec2::new(7.0, 17.0)),
                Transform::from_xyz(-6.0, -20.0, 0.2),
            ));
            player.spawn((
                Sprite::from_color(Color::srgb(0.11, 0.13, 0.14), Vec2::new(7.0, 17.0)),
                Transform::from_xyz(6.0, -20.0, 0.2),
            ));
        });

    commands.spawn((
        Camera2d,
        FollowCamera,
        Transform::from_xyz(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0),
    ));

    commands.spawn((
        Vignette,
        Sprite {
            image: asset_server.load("sprites/vignette.png"),
            custom_size: Some(Vec2::splat(1800.0)),
            ..default()
        },
        Transform::from_xyz(SPAWN.x, SPAWN.y, 50.0),
    ));
}

fn follow_camera(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut follow: ResMut<CameraFollow>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let dt = time.delta_secs();
    let target = Vec3::new(
        player.translation.x,
        player.translation.y + CAMERA_Y_OFFSET,
        0.0,
    );
    let blend = 1.0 - (-6.0 * dt).exp();
    follow.position = follow.position.lerp(target, blend);
}

fn add_shake_on_landing(
    mut events: MessageReader<LandingImpact>,
    mut shake: ResMut<CameraShake>,
) {
    for impact in events.read() {
        shake.trauma = (shake.trauma + impact.severity * 0.7).min(1.0);
    }
}

fn apply_camera_shake(
    time: Res<Time>,
    follow: Res<CameraFollow>,
    mut shake: ResMut<CameraShake>,
    mut camera: Query<&mut Transform, With<FollowCamera>>,
) {
    shake.trauma = (shake.trauma - SHAKE_DECAY_PER_SEC * time.delta_secs()).max(0.0);
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    if shake.trauma <= 0.0 {
        camera.translation = follow.position;
        return;
    }
    let power = shake.trauma * shake.trauma;
    let mut rng = rand::thread_rng();
    let jitter = Vec3::new(
        rng.gen_range(-1.0..1.0) * power * MAX_SHAKE_OFFSET,
        rng.gen_range(-1.0..1.0) * power * MAX_SHAKE_OFFSET,
        0.0,
    );
    camera.translation = follow.position + jitter;
}

fn follow_camera_with_vignette(
    camera: Query<&Transform, With<FollowCamera>>,
    mut vignette: Query<&mut Transform, (With<Vignette>, Without<FollowCamera>)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok(mut vignette) = vignette.single_mut() else {
        return;
    };
    vignette.translation.x = camera.translation.x;
    vignette.translation.y = camera.translation.y;
}

fn tick_spawn_hint(time: Res<Time>, mut hint: ResMut<SpawnHint>) {
    if hint.0 > 0.0 {
        hint.0 = (hint.0 - time.delta_secs()).max(0.0);
    }
}

fn reset_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut Body,
            &mut Survival,
            &mut PlayerInventory,
        ),
        With<Player>,
    >,
    mut stats: ResMut<RunStats>,
    mut selected: ResMut<SelectedSlot>,
    mut hint: ResMut<SpawnHint>,
    mut follow: ResMut<CameraFollow>,
    mut shake: ResMut<CameraShake>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    let Ok((mut transform, mut velocity, mut body, mut survival, mut inventory)) =
        query.single_mut()
    else {
        return;
    };
    transform.translation = SPAWN;
    *velocity = LinearVelocity::ZERO;
    body.0.clear();
    survival.0.reset();
    *inventory = PlayerInventory::default();
    *stats = RunStats::default();
    selected.0 = 0;
    hint.0 = HINT_SECONDS;
    follow.position = Vec3::new(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0);
    shake.trauma = 0.0;
}
