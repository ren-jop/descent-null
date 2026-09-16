//! Bevy wiring for two readable cave enemies: the slower crawler and the
//! faster silverfish/skitter. Both use the same small state machine so
//! difficulty rises without turning enemy code into a separate framework.

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::body::{landing_wound, Body, BodyRegion, DamageCause, LastDamageCause};
use crate::items::LastEvent;
use crate::player::Player;

use super::state::{state_for_distance, EnemyState, EnemyStats};

const PLAYER_ATTACK_DAMAGE: f32 = 5.0;
const MELEE_RANGE: f32 = 46.0;
const POISON_SECONDS: f32 = 6.0;
const POISON_TICK_INTERVAL: f32 = 1.0;
const POISON_DAMAGE_PER_TICK: f32 = 0.025;

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
            Self::Skitter => "SILVERFISH",
        }
    }
}

#[derive(Resource, Default)]
struct PoisonState {
    remaining: f32,
    until_tick: f32,
}

impl PoisonState {
    fn apply(&mut self) {
        self.remaining = POISON_SECONDS;
        self.until_tick = POISON_TICK_INTERVAL;
    }

    fn clear(&mut self) {
        self.remaining = 0.0;
        self.until_tick = 0.0;
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
        app.init_resource::<PoisonState>()
            .add_systems(Update, (enemy_ai, tick_poison, player_attack).chain());
    }
}

/// Existing world generation calls this generic spawner. Shallow layers stay
/// crawler-heavy, while deeper spawns have a chance to become a Silverfish.
pub fn spawn_enemy(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) -> Entity {
    let mut rng = rand::thread_rng();
    let kind = if pos.y < -1200.0 && rng.gen_bool(0.38) {
        EnemyKind::Skitter
    } else {
        EnemyKind::Crawler
    };
    spawn_kind(commands, asset_server, pos, kind)
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
    mut poison: ResMut<PoisonState>,
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
                    if enemy.kind == EnemyKind::Skitter {
                        poison.apply();
                        last.show("SILVERFISH BITE  POISONED - HEALTH WILL TICK");
                    } else {
                        last.show(format!("ATTACK: {}", enemy.kind.label()));
                    }
                }
            }
        }
    }
}

fn tick_poison(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut poison: ResMut<PoisonState>,
    mut player_body: Query<&mut Body, With<Player>>,
    mut cause: ResMut<LastDamageCause>,
    mut last: ResMut<LastEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        poison.clear();
        return;
    }
    if poison.remaining <= 0.0 {
        return;
    }

    let dt = time.delta_secs();
    poison.remaining = (poison.remaining - dt).max(0.0);
    poison.until_tick -= dt;

    if poison.until_tick <= 0.0 {
        poison.until_tick += POISON_TICK_INTERVAL;
        if let Ok(mut body) = player_body.single_mut() {
            body.0.apply_external_drain(POISON_DAMAGE_PER_TICK);
            cause.0 = DamageCause::Enemy;
        }

        // The initial poison warning stays visible for most of the effect.
        // Near the end, surface one explicit tick so the player connects the
        // stepped health loss with poison rather than assuming the HUD bugged.
        if poison.remaining <= 2.1 && last.remaining <= 0.20 {
            last.show("POISON TICK  HEALTH -2.5%");
        }
    }

    if poison.remaining <= 0.0 && last.remaining <= 0.20 {
        last.show("POISON CLEARED");
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
