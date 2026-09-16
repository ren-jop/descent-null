//! Bevy wiring for two readable cave enemies: the slower crawler and the
//! faster skitter. Both use the same small state machine so difficulty rises
//! without turning enemy code into a separate framework.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::{landing_wound, Body, BodyRegion, DamageCause, LastDamageCause};
use crate::items::LastEvent;
use crate::player::Player;

use super::state::{state_for_distance, EnemyState, EnemyStats};

const PLAYER_ATTACK_DAMAGE: f32 = 5.0;
const MELEE_RANGE: f32 = 46.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Crawler,
    Skitter,
}

impl EnemyKind {
    fn max_health(self) -> f32 {
        match self {
            Self::Crawler => 20.0,
            Self::Skitter => 12.0,
        }
    }

    fn chase_speed(self) -> f32 {
        match self {
            Self::Crawler => 90.0,
            Self::Skitter => 155.0,
        }
    }

    fn attack_cooldown(self) -> f32 {
        match self {
            Self::Crawler => 1.3,
            Self::Skitter => 0.8,
        }
    }

    fn attack_severity(self) -> f32 {
        match self {
            Self::Crawler => 0.5,
            Self::Skitter => 0.42,
        }
    }

    fn attack_region(self) -> BodyRegion {
        match self {
            Self::Crawler => BodyRegion::Torso,
            Self::Skitter => BodyRegion::LeftLeg,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Crawler => "CRAWLER",
            Self::Skitter => "SKITTER",
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub stats: EnemyStats,
    pub kind: EnemyKind,
    cooldown: f32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_ai, player_attack));
    }
}

pub fn spawn_enemy(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) -> Entity {
    spawn_kind(commands, asset_server, pos, EnemyKind::Crawler)
}

pub fn spawn_skitter(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) -> Entity {
    spawn_kind(commands, asset_server, pos, EnemyKind::Skitter)
}

fn spawn_kind(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, kind: EnemyKind) -> Entity {
    let mut entity = commands.spawn((
        Enemy {
            stats: EnemyStats::new(kind.max_health()),
            kind,
            cooldown: 0.0,
        },
        Transform::from_xyz(pos.x, pos.y, 0.4),
        RigidBody::Dynamic,
        Collider::rectangle(if kind == EnemyKind::Skitter { 20.0 } else { 24.0 }, 12.0),
        LockedAxes::ROTATION_LOCKED,
        ColliderDensity(1.5),
        Friction::new(0.9),
        GravityScale(1.5),
        LinearVelocity::ZERO,
    ));

    match kind {
        EnemyKind::Crawler => {
            entity.insert(Sprite {
                image: asset_server.load("sprites/enemy_crawler.png"),
                custom_size: Some(Vec2::new(27.0, 15.0)),
                ..default()
            });
        }
        EnemyKind::Skitter => {
            entity.insert(Sprite::from_color(
                Color::srgb(0.72, 0.22, 0.16),
                Vec2::new(22.0, 11.0),
            ));
        }
    }

    entity.id()
}

fn enemy_ai(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&Transform, &mut LinearVelocity, &mut Enemy), Without<Player>>,
    mut player_body: Query<&mut Body, With<Player>>,
    mut last: ResMut<LastEvent>,
    mut cause: ResMut<LastDamageCause>,
) {
    let Ok(player_transform) = player.single() else { return; };
    let Ok(mut body) = player_body.single_mut() else { return; };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (transform, mut velocity, mut enemy) in &mut enemies {
        let pos = transform.translation.truncate();
        let distance = pos.distance(player_pos);
        let mut state = state_for_distance(distance);

        // Skitters notice the player sooner than crawlers and therefore create
        // more pressure on deep ledges without needing a ranged attack.
        if enemy.kind == EnemyKind::Skitter && distance <= 430.0 && state == EnemyState::Idle {
            state = EnemyState::Chasing;
        }

        match state {
            EnemyState::Idle => velocity.x = 0.0,
            EnemyState::Chasing => {
                let direction = (player_pos.x - pos.x).signum();
                velocity.x = direction * enemy.kind.chase_speed();
            }
            EnemyState::Attacking => {
                velocity.x = 0.0;
                enemy.cooldown = (enemy.cooldown - dt).max(0.0);
                if enemy.cooldown <= 0.0 {
                    enemy.cooldown = enemy.kind.attack_cooldown();
                    if let Some(wound) = landing_wound(enemy.kind.attack_region(), enemy.kind.attack_severity()) {
                        body.0.apply_wound(wound);
                    }
                    cause.0 = DamageCause::Enemy;
                    last.show(format!("ATTACK: {}", enemy.kind.label()));
                }
            }
        }
    }
}

fn player_attack(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
    mut commands: Commands,
    mut last: ResMut<LastEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    let Ok(player_transform) = player.single() else { return; };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, mut enemy) in &mut enemies {
        if player_pos.distance(transform.translation.truncate()) > MELEE_RANGE { continue; }
        let label = enemy.kind.label();
        enemy.stats.take_damage(PLAYER_ATTACK_DAMAGE);
        if enemy.stats.is_dead() {
            commands.entity(entity).despawn();
            last.show(format!("{} DEFEATED", label));
        } else {
            last.show(format!("HIT {}", label));
        }
        break;
    }
}
