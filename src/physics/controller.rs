//! Dynamic character controller on Avian: acceleration, damping, slope-aware
//! grounding, coyote/buffer jumps, and landing-impact messages.

use avian2d::{math::*, prelude::*};
use bevy::ecs::query::Has;
use bevy::prelude::*;
use bevy::time::Virtual;

use super::jump::JumpAssist;
use super::landing::{landing_severity, FallTracker};

#[derive(Message, Clone, Debug)]
pub struct LandingImpact {
    pub entity: Entity,
    pub downward_speed: f32,
    pub severity: f32,
}

pub struct PhysicsGameplayPlugin;

impl Plugin for PhysicsGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>()
            .add_message::<LandingImpact>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    update_grounded,
                    tick_jump_and_landing,
                    apply_movement,
                    apply_movement_damping,
                )
                    .chain(),
            );
    }
}

#[derive(Message)]
pub enum MovementAction {
    Move(f32),
    Jump,
}

#[derive(Component)]
pub struct CharacterController;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

#[derive(Component)]
pub struct MovementAcceleration(pub f32);

#[derive(Component)]
pub struct MovementDampingFactor(pub f32);

#[derive(Component)]
pub struct JumpImpulse(pub f32);

#[derive(Component)]
pub struct MaxSlopeAngle(pub f32);

#[derive(Component, Default)]
pub struct JumpAssistState(pub JumpAssist);

#[derive(Component, Default)]
pub struct FallTrackerState(pub FallTracker);

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
    jump_assist: JumpAssistState,
    fall_tracker: FallTrackerState,
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(10.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            acceleration: MovementAcceleration(1250.0),
            damping: MovementDampingFactor(5.0),
            jump_impulse: JumpImpulse(600.0),
            max_slope_angle: MaxSlopeAngle(30.0_f32.to_radians()),
            jump_assist: JumpAssistState::default(),
            fall_tracker: FallTrackerState::default(),
        }
    }
}

fn keyboard_input(
    mut movement_writer: MessageWriter<MovementAction>,
    keyboard: Res<ButtonInput<KeyCode>>,
    virtual_time: Res<Time<Virtual>>,
) {
    // Full-screen UI pauses virtual time. Do not queue movement/jumps while a
    // player is reading the guide, viewing records or typing their expo name.
    if virtual_time.is_paused() {
        return;
    }

    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let direction = (right as i8 - left as i8) as f32;
    if direction != 0.0 {
        movement_writer.write(MovementAction::Move(direction));
    }
    if keyboard.just_pressed(KeyCode::Space) {
        movement_writer.write(MovementAction::Jump);
    }
}

fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_to(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });
        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn tick_jump_and_landing(
    time: Res<Time>,
    mut impacts: MessageWriter<LandingImpact>,
    mut query: Query<(
        Entity,
        &mut JumpAssistState,
        &mut FallTrackerState,
        &LinearVelocity,
        Has<Grounded>,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut assist, mut fall, velocity, grounded) in &mut query {
        assist.0.tick(dt, grounded);
        if let Some(speed) = fall.0.observe(grounded, velocity.y) {
            if let Some(severity) = landing_severity(speed) {
                impacts.write(LandingImpact {
                    entity,
                    downward_speed: speed,
                    severity,
                });
            }
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_reader: MessageReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut JumpAssistState,
        &mut LinearVelocity,
    )>,
) {
    let dt = time.delta_secs();
    let mut move_dir = 0.0_f32;
    let mut jump_pressed = false;
    for event in movement_reader.read() {
        match event {
            MovementAction::Move(direction) => move_dir += *direction,
            MovementAction::Jump => jump_pressed = true,
        }
    }
    let move_dir = move_dir.clamp(-1.0, 1.0);

    for (accel, jump, mut assist, mut velocity) in &mut controllers {
        if move_dir != 0.0 {
            velocity.x += move_dir * accel.0 * dt;
        }
        if jump_pressed {
            assist.0.press_jump();
        }
        if assist.0.consume_jump() {
            velocity.y = jump.0;
        }
    }
}

fn apply_movement_damping(
    time: Res<Time>,
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity), With<CharacterController>>,
) {
    let dt = time.delta_secs();
    for (damping, mut velocity) in &mut query {
        velocity.x *= 1.0 / (1.0 + damping.0 * dt);
    }
}
