//! bevy wiring for the cave crawler enemy: spawning, the idle/chase/
//! attack AI, and the player's melee attack (`E`). depends on `body`
//! (bites create a real wound through the same pipeline as fall damage)
//! and `player`/`items` (who to chase, where to report combat text).

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::{landing_wound, Body, BodyRegion};
use crate::items::LastEvent;
use crate::player::Player;

use super::state::{state_for_distance, EnemyState, EnemyStats};

const ENEMY_MAX_HEALTH: f32 = 20.0;
const CHASE_SPEED: f32 = 90.0;
const ATTACK_COOLDOWN: f32 = 1.3;
/// fed into body::landing_wound — 0.5 lands in the laceration range, so a
/// bite draws blood, same pipeline a bad landing uses.
const BITE_SEVERITY: f32 = 0.5;
const PLAYER_ATTACK_DAMAGE: f32 = 5.0;
const MELEE_RANGE: f32 = 46.0;

#[derive(Component)]
pub struct Enemy {
    pub stats: EnemyStats,
    cooldown: f32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_ai, player_attack));
    }
}

/// spawns one cave crawler at `pos`. called from world generation, not
/// from this plugin's own Startup — the cave owns where enemies land.
/// returns the entity so the caller can tag it (e.g. for despawning on
/// cave regeneration) without this module needing to know why.
pub fn spawn_enemy(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) -> Entity {
    commands
        .spawn((
            Enemy { stats: EnemyStats::new(ENEMY_MAX_HEALTH), cooldown: 0.0 },
            Sprite {
                image: asset_server.load("sprites/enemy_crawler.png"),
                custom_size: Some(Vec2::new(27.0, 15.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.4),
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 12.0),
            LockedAxes::ROTATION_LOCKED,
            ColliderDensity(1.5),
            Friction::new(0.9),
            GravityScale(1.5),
            LinearVelocity::ZERO,
        ))
        .id()
}

fn enemy_ai(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&Transform, &mut LinearVelocity, &mut Enemy), Without<Player>>,
    mut player_body: Query<&mut Body, With<Player>>,
    mut last: ResMut<LastEvent>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut body) = player_body.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (transform, mut velocity, mut enemy) in &mut enemies {
        let pos = transform.translation.truncate();
        let distance = pos.distance(player_pos);

        match state_for_distance(distance) {
            EnemyState::Idle => velocity.x = 0.0,
            EnemyState::Chasing => {
                let direction = (player_pos.x - pos.x).signum();
                velocity.x = direction * CHASE_SPEED;
            }
            EnemyState::Attacking => {
                velocity.x = 0.0;
                enemy.cooldown = (enemy.cooldown - dt).max(0.0);
                if enemy.cooldown <= 0.0 {
                    enemy.cooldown = ATTACK_COOLDOWN;
                    if let Some(wound) = landing_wound(BodyRegion::Torso, BITE_SEVERITY) {
                        body.0.apply_wound(wound);
                    }
                    last.show("bitten — bleeding from torso");
                }
            }
        }
    }
}

/// `E` — melee attack. hits the first enemy in range, once per press.
/// no swing animation/hitbox — a proximity check, matching how pickups
/// and supply use already work in this codebase.
fn player_attack(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
    mut commands: Commands,
    mut last: ResMut<LastEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, mut enemy) in &mut enemies {
        if player_pos.distance(transform.translation.truncate()) > MELEE_RANGE {
            continue;
        }
        enemy.stats.take_damage(PLAYER_ATTACK_DAMAGE);
        if enemy.stats.is_dead() {
            commands.entity(entity).despawn();
            last.show("enemy defeated");
        } else {
            last.show(format!("hit enemy ({:.0}/{:.0} hp)", enemy.stats.health(), enemy.stats.max_health()));
        }
        break;
    }
}
