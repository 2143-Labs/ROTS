use crate::{
    setup::CameraFollow,
    sprites::AnimationTimer,
    states::{FreeCamState, GameState},
};
use bevy::{
    input::mouse::MouseWheel, prelude::*, render::render_resource::BindGroupLayoutDescriptor,
};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_rapier3d::prelude::{Collider, ExternalImpulse, GravityScale, LockedAxes, RigidBody};
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};

pub fn init(app: &mut App) -> &mut App {
    app.add_system(spawn_player_sprite.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_systems(
            (player_movement, camera_follow_system)
                .distributive_run_if(in_state(FreeCamState::Locked)),
        )
}

#[derive(Component)]
pub struct PlayerCamera; // tag entity to make it always face the camera

#[derive(AssetCollection, Resource)]
pub struct PlayerSpriteAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    #[asset(texture_atlas(columns = 3, rows = 1))]
    #[asset(path = "brownSheet.png")]
    pub run: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct FaceCamera; // tag entity to make it always face the camera

#[derive(Reflect, Component)]
pub struct Player {
    pub looking_at: Vec3,
    pub facing_vel: f32,
    pub position: Vec3,
    pub velocity: Vec3,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn spawn_player_sprite(
    mut commands: Commands,
    images: Res<PlayerSpriteAssets>,
    mut sprite_params: Sprite3dParams,
) {
    let sprite = AtlasSprite3d {
        atlas: images.run.clone(),

        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,

        index: 1,

        transform: Transform::from_xyz(-3., 0.5, 2.).looking_at(Vec3::new(10., 10., 10.), Vec3::Y),
        // pivot: Some(Vec2::new(0.5, 0.5)),
        ..default()
    }
    .bundle(&mut sprite_params);

    commands
        .spawn((sprite, RigidBody::Dynamic))
        .insert(Name::new("PlayerSprite"))
        .insert(Player::default())
        .insert(FaceCamera)
        .insert(AnimationTimer(Timer::from_seconds(
            0.4,
            TimerMode::Repeating,
        )))
        // .insert(LockedAxes::ROTATION_LOCKED)
        .insert(GravityScale(1.))
        .insert(ExternalImpulse {
            impulse: Vec3::new(1.0, 8.0, 2.0),
            torque_impulse: Vec3::new(0.4, 0.4, 0.4),
        })
        .insert(Collider::cuboid(0.1, 1., 1.));
}

pub const PLAYER_SPEED: f32 = 5.;
pub fn player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    _camera_query: Query<&Transform, (With<CameraFollow>, Without<Player>)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let rotation = Vec3::ONE;
    // if let Ok(transform) = camera_query.get_single(){
    //     rotation = transform.rotation * Vec3::ONE;
    // }
    if let Ok(mut transform) = player_query.get_single_mut() {
        //Get cameras facing vector
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            direction += rotation * Vec3::new(-1., 0., -1.);
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction += rotation * Vec3::new(1., 0., 1.);
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction += rotation * Vec3::new(-1., 0., 1.);
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += rotation * Vec3::new(1., 0., -1.);
        }
        if direction.length() > 0. {
            direction = direction.normalize();
        }
        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

pub fn camera_follow_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
) {
    for event in mouse_wheel_events.iter() {
        for (_, mut camera_follow) in camera_query.iter_mut() {
            camera_follow.distance = match event.y {
                y if y < 0. => (camera_follow.distance + 1.).abs(),
                y if y > 0. => (camera_follow.distance - 1.).abs(),
                _ => camera_follow.distance,
            };
            if camera_follow.distance < camera_follow.min_distance {
                camera_follow.distance = camera_follow.min_distance;
            } else if camera_follow.distance > camera_follow.max_distance {
                camera_follow.distance = camera_follow.max_distance;
            }
        }
    }
    if let Ok(player_transform) = player_query.get_single() {
        for (mut transform, camera_follow) in camera_query.iter_mut() {
            transform.translation =
                Vec3::new(1., 1., 1.) * camera_follow.distance + player_transform.translation;
            //.looking_at(Vec3::new(10., 10., 10.), Vec3::Y);
        }
    }
}
