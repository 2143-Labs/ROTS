use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::CursorGrabMode,
};
use bevy_fly_camera::FlyCamera;
use bevy_mod_raycast::RaycastSource;
use shared::Config;

use crate::{
    player::Player,
    setup::MyRaycastSet,
    states::{CameraState, GameState},
};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_startup_system(switch_camera_state)
        .add_system(setup_camera.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_system(wow_camera_system
                .run_if(in_state(CameraState::ThirdPersonLocked).or_else(in_state(CameraState::ThirdPersonFreeMouse))))
        .add_system(q_e_rotate_cam
                .run_if(in_state(CameraState::ThirdPersonFreeMouse)))
         .add_system(
            camera_pan_system.run_if(
                in_state(CameraState::ThirdPersonFreeMouse)
                    .or_else(in_state(CameraState::ThirdPersonLocked)),
        ))
        .add_system(camera_topdown_system.run_if(in_state(CameraState::TopDown)))
}
#[derive(Reflect, Component)]
pub struct PlayerCamera {
    // tag entity to make it always face the camera
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub dragging: bool,
    pub degrees: f32,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub old_degrees: f32,
}
impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            ..default()
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
enum ProjectionType {
    #[default]
    Perspective,
    Orthographic,
}
// TODO: Fix this
fn swap_projection(_camera: &Mut<Camera3d>, orthographic: ProjectionType) {
    // Create a new camera entity and add a transform component to it
    // let mut camera_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 10.0));
    // camera_transform.look_at(Vec3::default(), Vec3::Y);
    let _proj = match orthographic {
        ProjectionType::Perspective => Projection::Perspective(PerspectiveProjection {
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            // aspect_ratio: 1.0,
            ..Default::default()
        }),
        ProjectionType::Orthographic => Projection::Orthographic(OrthographicProjection {
            near: 0.0,
            far: 100.0,
            ..Default::default()
        }),
    };
    // Add the camera component to the entity
    // Camera3d.projection = proj;
}

pub fn camera_topdown_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single_mut() {
        for (mut cam_transform, mut camera_follow) in camera_query.iter_mut() {
            // Scroll wheel zoom
            for event in mouse_wheel_events.iter() {
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
            // Stay above player
            let new_transform = Transform::from_translation(
                player_transform.translation + Vec3::new(0., camera_follow.distance, 0.),
            )
            .looking_at(player_transform.translation, Vec3::Z);
            cam_transform.translation = new_transform.translation;
            cam_transform.rotation = new_transform.rotation;
        }
    }
}

pub fn camera_pan_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<Input<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Ok((player_transform, mut player)) = player_query.get_single_mut() {
        for (_, mut camera_follow) in camera_query.iter_mut() {
            //dbg!{player.lock_movement};
            for event in mouse_wheel_events.iter() {
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
            if mouse_input.pressed(MouseButton::Right) {
                camera_follow.dragging = true;
                for event in mouse_events.iter() {
                    camera_follow.degrees += event.delta.x;
                }
                // player.lock_movement = [None; 4];
            }
            if mouse_input.just_released(MouseButton::Right) {
                camera_follow.dragging = false;
                camera_follow.old_degrees = camera_follow.degrees;
                player.lock_movement = [None; 4];
                if keyboard_input.pressed(KeyCode::W) {
                    player.lock_movement[0] = Some(Vec2::new(
                        f32::to_radians(camera_follow.degrees).sin(),
                        f32::to_radians(camera_follow.degrees).cos(),
                    ));
                }
                if keyboard_input.pressed(KeyCode::S) {
                    player.lock_movement[2] = Some(Vec2::new(
                        f32::to_radians(camera_follow.degrees).sin(),
                        f32::to_radians(camera_follow.degrees).cos(),
                    ));
                }
            }
        }
        for (mut transform, camera_follow) in camera_query.iter_mut() {
            let new_transform = Transform::from_translation(
                Vec3::new(
                    f32::to_radians(camera_follow.degrees).sin(),
                    1.,
                    f32::to_radians(camera_follow.degrees).cos(),
                ) * camera_follow.distance
                    + player_transform.translation,
            )
            .looking_at(player_transform.translation, Vec3::Y);
            transform.translation = new_transform.translation;
            transform.rotation = new_transform.rotation;
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(RaycastSource::<MyRaycastSet>::new_transform_empty())
        .insert(PlayerCamera::default())
        .insert(Name::new("Camera"));
}

pub fn switch_camera_state(
    mut players: Query<Entity, With<Player>>,
    mut commands: Commands,
    cam_state: Res<State<CameraState>>,
    mut next_state: ResMut<NextState<CameraState>>,
    mut cam_query: Query<&mut Camera3d>,
    input: Res<Input<KeyCode>>,
    mut windows_query: Query<&mut Window>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = windows_query.get_single_mut() {
            window.cursor.grab_mode = match window.cursor.grab_mode {
                CursorGrabMode::None => CursorGrabMode::Locked,
                CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
            };
            window.cursor.visible = !window.cursor.visible;
        };
        //info!(?cam_state.0);
        next_state.set(match cam_state.0 {
            CameraState::ThirdPersonFreeMouse => {
                for player in players.iter_mut() {
                    commands.entity(player).insert(FlyCamera::default());
                }
                CameraState::Free
            }
            CameraState::Free => {
                for player in players.iter_mut() {
                    commands.entity(player).remove::<FlyCamera>();
                }
                CameraState::ThirdPersonLocked
            }
            CameraState::ThirdPersonLocked => {
                if let Ok(cam) = cam_query.get_single_mut() {
                    swap_projection(&cam, ProjectionType::Orthographic);
                }
                CameraState::TopDown
            }
            CameraState::TopDown => {
                if let Ok(cam) = cam_query.get_single_mut() {
                    swap_projection(&cam, ProjectionType::Perspective);
                }
                CameraState::ThirdPersonFreeMouse
            }
        });
    }
}

pub fn q_e_rotate_cam(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut PlayerCamera>,
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
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    camera_type: Res<State<CameraState>>,
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
            || camera_type.0 == CameraState::ThirdPersonLocked
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
