use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};
use shared::AnyPlayer;

use crate::worldgen::ChunkPos;

#[derive(Reflect, Component)]
pub struct Player {
    pub looking_at: Vec3,
    pub facing_vel: f32,
    pub velocity: Vec3,
    pub lock_movement: [Option<Vec2>; 4],
    pub current_chunk: ChunkPos,
}

#[derive(Reflect, Component)]
pub struct PlayerName(pub String);

#[derive(Component)]
pub struct MovementIntention(pub Vec2);

impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            velocity: Vec3::ZERO,
            lock_movement: [None; 4],
            current_chunk: ChunkPos(0, 0, 0),
        }
    }
}
#[derive(Resource)]
pub struct Animation(Handle<AnimationClip>);

pub fn spawn_player_sprite(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    // let cube = PbrBundle {
    // mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    // material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
    // transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    // ..Default::default()
    // };

    commands.insert_resource(Animation(asset_server.load("tadpole.gltf#Animation0")));

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("tadpole.gltf#Scene0"),
            transform: Transform::from_xyz(0., 1.0, 0.)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                .with_scale(Vec3::new(4., 4., 4.)),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(1., 1., 1.),
        Name::new("Player"),
        MovementIntention(Vec2::ZERO),
        Player::default(),
        crate::cameras::FaceCamera,
        crate::physics::Jumper {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
        AnyPlayer,
    ));
}

// Once the scene is loaded, start the animation
pub fn animate_sprites(
    animations: Res<Animation>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0.clone_weak()).repeat();
    }
}
