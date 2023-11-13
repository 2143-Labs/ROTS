use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};

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
            ..default()
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
        crate::cameras::FaceCamera,
        crate::physics::Jumper {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
    ));
}
