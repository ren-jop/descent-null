//! Player spawn, camera follow, and proving-ground reset.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::items::PlayerInventory;
use crate::physics::CharacterControllerBundle;
use crate::survival::Survival;
use crate::world::RunStats;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (follow_camera, follow_camera_with_vignette, reset_player).chain());
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FollowCamera;

#[derive(Component)]
struct Vignette;

const SPAWN: Vec3 = Vec3::new(-420.0, 40.0, 0.0);

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

    commands.spawn((Camera2d, FollowCamera, Transform::from_xyz(SPAWN.x, SPAWN.y, 0.0)));

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
}

fn follow_camera(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let target = Vec3::new(player.translation.x, player.translation.y + 48.0, camera.translation.z);
    let blend = 1.0 - (-6.0 * dt).exp();
    camera.translation = camera.translation.lerp(target, blend);
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

fn reset_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (&mut Transform, &mut LinearVelocity, &mut Body, &mut Survival, &mut PlayerInventory),
        With<Player>,
    >,
    mut stats: ResMut<RunStats>,
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
}
