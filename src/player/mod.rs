//! Player spawn, camera follow, and proving-ground reset.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::physics::CharacterControllerBundle;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (follow_camera, reset_player));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FollowCamera;

const SPAWN: Vec3 = Vec3::new(-420.0, 40.0, 0.0);

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Capsule2d::new(12.5, 20.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.86, 0.72, 0.28))),
        Transform::from_translation(SPAWN),
        CharacterControllerBundle::new(Collider::capsule(12.5, 20.0)),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        ColliderDensity(2.0),
        GravityScale(1.5),
        TransformInterpolation,
        Body::default(),
    ));

    commands.spawn((Camera2d, FollowCamera, Transform::from_xyz(SPAWN.x, SPAWN.y, 0.0)));
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

fn reset_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut LinearVelocity, &mut Body), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    let Ok((mut transform, mut velocity, mut body)) = query.single_mut() else {
        return;
    };
    transform.translation = SPAWN;
    *velocity = LinearVelocity::ZERO;
    body.0.clear();
}
