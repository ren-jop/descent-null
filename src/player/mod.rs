//! Player spawn, camera follow (with landing-triggered shake), nametag,
//! spawn-hint timer, and reset.

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::body::Body;
use crate::items::PlayerInventory;
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
                    follow_nametag,
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

#[derive(Component)]
struct Nametag;

/// how long the on-screen control hints stay visible after a (re)spawn.
const HINT_SECONDS: f32 = 6.0;

/// seconds left to show the "how to play" hint. counts down to 0; ui
/// reads this to decide whether to show it. reset to HINT_SECONDS on
/// every respawn (and starts at HINT_SECONDS on first spawn too).
#[derive(Resource)]
pub struct SpawnHint(pub f32);

impl Default for SpawnHint {
    fn default() -> Self {
        Self(HINT_SECONDS)
    }
}

const SPAWN: Vec3 = Vec3::new(-420.0, 40.0, 0.0);
const NAMETAG_OFFSET: f32 = 30.0;
const CAMERA_Y_OFFSET: f32 = 48.0;

/// the camera's smoothed target position, tracked separately from the
/// actual rendered Transform. apply_camera_shake writes
/// `position + jitter` into the real Transform every frame instead of
/// jittering the Transform directly — otherwise next frame's follow-lerp
/// would start from an already-shaken position and the offset would
/// partially compound/drift instead of settling cleanly.
#[derive(Resource)]
struct CameraFollow {
    position: Vec3,
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self { position: Vec3::new(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0) }
    }
}

/// 0..1 "how much screen shake right now", decaying over time. bumped by
/// hard landings (see add_shake_on_landing).
#[derive(Resource, Default)]
struct CameraShake {
    trauma: f32,
}

const SHAKE_DECAY_PER_SEC: f32 = 1.4;
const MAX_SHAKE_OFFSET: f32 = 18.0;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite {
            image: asset_server.load("sprites/player.png"),
            custom_size: Some(Vec2::new(25.0, 40.0)),
            ..default()
        },
        Transform::from_translation(SPAWN),
        CharacterControllerBundle::new(Collider::capsule(12.5, 20.0)),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        ColliderDensity(2.0),
        GravityScale(1.5),
        TransformInterpolation,
        Body::default(),
        Survival::default(),
        PlayerInventory::default(),
    ));

    commands.spawn((Camera2d, FollowCamera, Transform::from_xyz(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0)));

    // atmosphere: a large soft vignette that rides on the camera, always
    // centered on screen, drawn above everything else.
    commands.spawn((
        Vignette,
        Sprite {
            image: asset_server.load("sprites/vignette.png"),
            custom_size: Some(Vec2::splat(1800.0)),
            ..default()
        },
        Transform::from_xyz(SPAWN.x, SPAWN.y, 50.0),
    ));

    // world-space nametag, floating above the character.
    commands.spawn((
        Nametag,
        Text2d::new("Player One"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.92, 0.90, 0.84)),
        Transform::from_xyz(SPAWN.x, SPAWN.y + NAMETAG_OFFSET, 1.0),
    ));
}

fn follow_camera(time: Res<Time>, player: Query<&Transform, With<Player>>, mut follow: ResMut<CameraFollow>) {
    let Ok(player) = player.single() else {
        return;
    };
    let dt = time.delta_secs();
    let target = Vec3::new(player.translation.x, player.translation.y + CAMERA_Y_OFFSET, 0.0);
    let blend = 1.0 - (-6.0 * dt).exp();
    follow.position = follow.position.lerp(target, blend);
}

/// hard landings punch the camera — a bit of physical feedback for the
/// exact moment that also causes real damage, per the "make it feel more
/// impactful" direction. severity is already 0..1 (see physics::landing).
fn add_shake_on_landing(mut events: MessageReader<LandingImpact>, mut shake: ResMut<CameraShake>) {
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
    // eased falloff — shake feels punchy at first, tapers quickly.
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

fn follow_nametag(
    player: Query<&Transform, (With<Player>, Without<Nametag>)>,
    mut nametag: Query<&mut Transform, With<Nametag>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut nametag) = nametag.single_mut() else {
        return;
    };
    nametag.translation.x = player.translation.x;
    nametag.translation.y = player.translation.y + NAMETAG_OFFSET;
}

fn tick_spawn_hint(time: Res<Time>, mut hint: ResMut<SpawnHint>) {
    if hint.0 > 0.0 {
        hint.0 = (hint.0 - time.delta_secs()).max(0.0);
    }
}

fn reset_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (&mut Transform, &mut LinearVelocity, &mut Body, &mut Survival, &mut PlayerInventory),
        With<Player>,
    >,
    mut stats: ResMut<RunStats>,
    mut hint: ResMut<SpawnHint>,
    mut follow: ResMut<CameraFollow>,
    mut shake: ResMut<CameraShake>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    let Ok((mut transform, mut velocity, mut body, mut survival, mut inventory)) = query.single_mut()
    else {
        return;
    };
    transform.translation = SPAWN;
    *velocity = LinearVelocity::ZERO;
    body.0.clear();
    survival.0.reset();
    *inventory = PlayerInventory::default();
    *stats = RunStats::default();
    hint.0 = HINT_SECONDS;
    follow.position = Vec3::new(SPAWN.x, SPAWN.y + CAMERA_Y_OFFSET, 0.0);
    shake.trauma = 0.0;
}
