use std::f32::consts::PI;

use crate::{
    setup::CameraFollow,
    sprites::AnimationTimer,
    states::{FreeCamState, GameState},
};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, ExternalImpulse, GravityScale, LockedAxes, RigidBody,
};
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};
use shared::Config;

pub fn init(app: &mut App) -> &mut App {
    app.add_systems(OnEnter(GameState::Ready), spawn_player_sprite)
        .add_systems(
            Update,
            (player_movement, wow_camera_system)
                .distributive_run_if(in_state(FreeCamState::ThirdPersonLocked)),
        )
        .add_systems(
            Update,
            (player_movement, wow_camera_system, q_e_rotate_cam)
                .distributive_run_if(in_state(FreeCamState::ThirdPersonFreeMouse)),
        )
        .register_type::<Jumper>()
}

#[derive(Reflect, Component)]
pub struct Jumper {
    //pub cooldown: f32,
    pub timer: Timer,
}

//#[derive(Resource)]
//pub struct PlayerSpriteAssets {
    //#[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    //#[asset(texture_atlas(columns = 3, rows = 1))]
    //#[asset(path = "brownSheet.png")]
    //pub run: Handle<TextureAtlas>,
//}

#[derive(Component)]
pub struct FaceCamera; // tag entity to make it always face the camera

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
    images: Res<AssetServer>,
    mut sprite_params: Sprite3dParams,
) {
    let texture_handle = images.load("brownSheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 1, None, None);

    let starting_location = Vec3::new(-3., 0.5, 2.);
    let sprite = AtlasSprite3d {
        atlas: texture_atlas,

        pixels_per_metre: 32.,
        alpha_mode: AlphaMode::Add,
        unlit: true,

        index: 1,

        transform: Transform::from_translation(starting_location),
        // pivot: Some(Vec2::new(0.5, 0.5)),
        ..default()
    }
    .bundle(&mut sprite_params);

    commands.spawn((
        sprite,
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.2),
        LockedAxes::ROTATION_LOCKED,
        GravityScale(1.),
        ColliderMassProperties::Mass(1.0),
        Name::new("PlayerSprite"),
        Player::default(),
        FaceCamera,
        Jumper {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
        AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
    ));
}

pub const PLAYER_SPEED: f32 = 5.;
pub fn player_movement(
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, Entity, &mut Jumper, &mut Player)>,
    camera_query: Query<&CameraFollow>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, player_ent, mut jumper, _player) in player_query.iter_mut() {
        let mut move_vector = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            move_vector += Vec2::new(1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_vector += Vec2::new(-1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_vector += Vec2::new(0.0, -1.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_vector += Vec2::new(0.0, 1.0);
        }

        jumper.timer.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if jumper.timer.finished() {
                commands.entity(player_ent).insert(ExternalImpulse {
                    impulse: Vec3::new(0., 4., 0.),
                    torque_impulse: Vec3::new(0., 0., 0.),
                });
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

pub fn q_e_rotate_cam(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut CameraFollow>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let mut rotation = 0.0;
    if keyboard_input.pressed(KeyCode::Q) {
        rotation += 1.0;
    }
    if keyboard_input.pressed(KeyCode::E) {
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
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    _keyboard_input: Res<Input<KeyCode>>,
    camera_type: Res<State<FreeCamState>>,
    config: Res<Config>,
) {
    let player_transform = match player_query.get_single() {
        Ok(s) => s,
        Err(_) => return,
    };

    for (mut camera_transform, mut camera_follow) in camera_query.iter_mut() {
        for event in mouse_wheel_events.iter() {
            camera_follow.distance -= event.y;
            camera_follow.distance = camera_follow
                .distance
                .clamp(camera_follow.min_distance, camera_follow.max_distance);
        }

        if mouse_input.pressed(MouseButton::Right)
            || *camera_type == FreeCamState::ThirdPersonLocked
        {
            for event in mouse_events.iter() {
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
