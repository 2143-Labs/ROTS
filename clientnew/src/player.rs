use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};
use shared::Config;

use crate::cameras::*;

#[derive(Reflect, Component)]
pub struct Player {
    pub looking_at: Vec3,
    pub facing_vel: f32,
    pub velocity: Vec3,
    pub lock_movement: [Option<Vec2>; 4],
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            velocity: Vec3::ZERO,
            lock_movement: [None; 4],
        }
    }
}

pub fn spawn_player_sprite(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        ..Default::default()
    };

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1., 1., 1.),
        cube,
        Name::new("Player"),
        Player::default(),
        FaceCamera,
        Jumper {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
    ));
}

pub const PLAYER_SPEED: f32 = 5.;
pub fn player_movement(
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, Entity, &mut Jumper, &mut Player)>,
    camera_query: Query<&CameraFollow>,
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    for (mut transform, player_ent, mut jumper, _player) in player_query.iter_mut() {
        let mut move_vector = Vec2::ZERO;
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::MoveForward,
        ) {
            move_vector += Vec2::new(1.0, 0.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::MoveBackward,
        ) {
            move_vector += Vec2::new(-1.0, 0.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::StrafeLeft,
        ) {
            move_vector += Vec2::new(0.0, -1.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::StrafeRight,
        ) {
            move_vector += Vec2::new(0.0, 1.0);
        }

        jumper.timer.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if jumper.timer.finished() {
                // TODO jump
                jumper.timer.reset();
            }
        }

        if move_vector.length_squared() > 0.0 {
            let camera = camera_query.single();
            let rotation = Vec2::from_angle(-camera.yaw_radians);
            let movem = move_vector.normalize().rotate(rotation);

            transform.translation +=
                Vec3::new(movem.x, 0.0, movem.y) * PLAYER_SPEED * time.delta_seconds();
        }
    }
}
