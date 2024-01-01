use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use shared::{
    animations::AnimationTimer, event::server::Cast, unit::MovementIntention, Config, GameAction,
};

use crate::{
    physics::Jumper,
    player::{Player, PrimaryUnitControl},
};

use super::{ClientAimDirection, FreeCamState};

#[derive(Reflect, Component)]
pub struct CameraFollow {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub dragging: bool,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub old_yaw: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            dragging: false,
            yaw_radians: 0.,
            pitch_radians: PI * 1.0 / 4.0,
            old_yaw: 0.,
        }
    }
}

pub fn q_e_rotate_cam(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut CameraFollow>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let mut rotation = 0.0;

    if config.pressed(&keyboard_input, GameAction::RotateLeft) {
        rotation += 1.0;
    }
    if config.pressed(&keyboard_input, GameAction::RotateRight) {
        rotation -= 1.0;
    }
    if rotation != 0.0 {
        camera_query.single_mut().yaw_radians += config.qe_sens * rotation * time.delta_seconds();
    }
}

pub fn wow_camera_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<Input<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<Camera3d>>,
    mut client_aim_direction: Query<&mut ClientAimDirection>,
    current_unit: Query<&Transform, (With<PrimaryUnitControl>, Without<CameraFollow>)>,
    _keyboard_input: Res<Input<KeyCode>>,
    camera_type: Res<State<FreeCamState>>,
    config: Res<Config>,
) {
    let player_transform = match current_unit.get_single() {
        Ok(s) => s,
        Err(_) => return,
    };

    for (mut camera_transform, mut camera_follow) in camera_query.iter_mut() {
        // Update the direction we will shoot in.
        client_aim_direction.single_mut().0 = camera_follow.yaw_radians;

        for event in mouse_wheel_events.read() {
            camera_follow.distance -= event.y;
            camera_follow.distance = camera_follow
                .distance
                .clamp(camera_follow.min_distance, camera_follow.max_distance);
        }

        for event in mouse_events.read() {
            //https://github.com/bevyengine/bevy/issues/10860
            if *camera_type == FreeCamState::ThirdPersonLocked
                || mouse_input.pressed(MouseButton::Right)
            {
                let sens = config.sens;
                camera_follow.yaw_radians -= event.delta.x * sens;
                camera_follow.pitch_radians -= event.delta.y * sens;
                camera_follow.pitch_radians =
                    camera_follow.pitch_radians.clamp(0.05 * PI, 0.95 * PI);
            }
        }

        let camera_location = Quat::from_rotation_y(camera_follow.yaw_radians)
            * Quat::from_rotation_z(camera_follow.pitch_radians)
            * Vec3::Y
            * camera_follow.distance
            + player_transform.translation;

        let new_transform = Transform::from_translation(camera_location)
            .looking_at(player_transform.translation + 0.5 * Vec3::Y, Vec3::Y);

        camera_transform.translation = new_transform.translation;
        camera_transform.rotation = new_transform.rotation;
    }
}

#[derive(Debug)]
pub(crate) struct LastMovement(pub Vec2);

impl Default for LastMovement {
    fn default() -> Self {
        Self(Vec2::new(1.0, 0.0))
    }
}

pub const PLAYER_SPEED: f32 = 25.;
pub(crate) fn player_movement(
    _commands: Commands,
    mut player_query: Query<(
        &mut Transform,
        Entity,
        &mut Jumper,
        &mut Player,
        &mut MovementIntention,
        Option<(&AnimationTimer, &Cast)>,
    )>,
    camera_query: Query<&CameraFollow>,
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    mut last_movement: Local<LastMovement>,
    time: Res<Time>,
) {
    for (mut transform, _player_ent, mut jumper, _player, mut movement, casting) in
        player_query.iter_mut()
    {
        let mut move_vector = Vec2::ZERO;
        if config.pressed(&keyboard_input, GameAction::MoveForward) {
            move_vector += Vec2::new(1.0, 0.0);
        }
        if config.pressed(&keyboard_input, GameAction::MoveBackward) {
            move_vector += Vec2::new(-1.0, 0.0);
        }
        if config.pressed(&keyboard_input, GameAction::StrafeLeft) {
            move_vector += Vec2::new(0.0, -1.0);
        }
        if config.pressed(&keyboard_input, GameAction::StrafeRight) {
            move_vector += Vec2::new(0.0, 1.0);
        }

        jumper.timer.tick(time.delta());
        if config.pressed(&keyboard_input, GameAction::Jump) {
            if jumper.timer.finished() {
                // TODO does this reset with the overflow from the extra time.delta()?
                // or are we really jumping at slightly slower rate?
                jumper.timer.reset();
            }
        }

        let final_move = if move_vector.length_squared() > 0.0 {
            let camera = camera_query.single();
            let rotation = Vec2::from_angle(-camera.yaw_radians);
            let mut movem = move_vector.normalize().rotate(rotation);

            if let Some((anim_timer, cast)) = casting {
                let anim_timer = &anim_timer.0;
                let anim = cast.get_current_animation(anim_timer.elapsed());
                match anim {
                    shared::animations::AnimationState::FrontSwing => movem *= 0.75,
                    shared::animations::AnimationState::WindUp => movem *= 0.25,
                    shared::animations::AnimationState::WindDown => movem *= 0.5,
                    shared::animations::AnimationState::Backswing => movem *= 0.75,
                    shared::animations::AnimationState::Done => {}
                }
            }

            transform.translation +=
                Vec3::new(movem.x, 0.0, movem.y) * PLAYER_SPEED * time.delta_seconds();

            last_movement.0 = movem;

            movem
        } else {
            move_vector
        };

        // point in the direction you are moving, offset by (animation sections * 1 turn per second)
        let movem = last_movement.0;
        transform.rotation = Quat::from_rotation_y(movem.x.atan2(movem.y));

        if let Some((anim_timer, cast)) = casting {
            let anim_timer = &anim_timer.0;
            let anim = cast.get_current_animation(anim_timer.elapsed());
            let time_offset = anim_timer.elapsed_secs() * PI * 2.0;

            match anim {
                shared::animations::AnimationState::FrontSwing => {
                    // A forward spin
                    transform.rotation *= Quat::from_rotation_x(time_offset);
                }
                shared::animations::AnimationState::WindUp => {
                    // spinning in all directions
                    transform.rotation *= Quat::from_rotation_z(time_offset);
                    transform.rotation *= Quat::from_rotation_y(time_offset);
                    transform.rotation *= Quat::from_rotation_x(time_offset);
                }
                shared::animations::AnimationState::WindDown => {
                    // flipped upside down like a turtle
                    transform.rotation *= Quat::from_rotation_x(PI);
                    transform.rotation *= Quat::from_rotation_y(time_offset);
                    transform.rotation *= Quat::from_rotation_z(time_offset.sin());
                }
                shared::animations::AnimationState::Backswing => {
                    // slowly turn back up
                    let si = cast.get_skill_info();
                    let pct_recovered = (anim_timer.elapsed() - si.get_free_point()).as_secs_f32() / (si.get_total_duration() - si.get_free_point()).as_secs_f32();
                    transform.rotation *= Quat::from_rotation_x(PI);
                    transform.rotation *= Quat::from_rotation_x(pct_recovered * PI);
                }
                shared::animations::AnimationState::Done => {} // no rotation
            }
        };

        let y = jumper.get_y() + 1.0;
        if transform.translation.y != y {
            transform.translation.y = y
        }

        // only change this if we have to. This will trigger a packet to be sent
        if movement.0 != final_move {
            movement.0 = final_move;
        }
    }
}
