use std::f32::consts::PI;
use bevy::{prelude::*, input::mouse::{MouseWheel, MouseMotion}};
use bevy_mod_raycast::RaycastSource;
use shared::Config;

use crate::{setup::MyRaycastSet, player::Player, states::FreeCamState};
pub fn init(app: &mut App) -> &mut App {
    app.add_startup_system(spawn_camera)
        .add_system( wow_camera_system
                .run_if(in_state(FreeCamState::ThirdPersonLocked)
               .or_else(in_state(FreeCamState::ThirdPersonFreeMouse))))
        .add_system(camera_zoom_system
                .run_if(in_state(FreeCamState::TopDown)
               .or_else(in_state(FreeCamState::ThirdPersonFreeMouse))))
        .add_system(q_e_rotate_cam
                .run_if(in_state(FreeCamState::ThirdPersonFreeMouse)
               .or_else(in_state(FreeCamState::TopDown))))
        .add_system(camera_topdown_system
                .run_if(in_state(FreeCamState::TopDown)
                ))
}

#[derive(Reflect, Component)]
pub struct PlayerCamera {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub dragging: bool,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub old_yaw: f32,
}
impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            dragging: false,
            yaw_radians: 0.,
            pitch_radians: PI * 1.0/4.0,
            old_yaw: 0.,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            RaycastSource::<MyRaycastSet>::new_transform_empty(),
        ))
        .insert(PlayerCamera::default())
        .insert(Name::new("Camera"))
        ;
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
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    camera_type: Res<State<FreeCamState>>,
    config: Res<Config>,
) {
    let player_transform = match player_query.get_single() {
        Ok(s) => s,
        Err(_) => return,
    };

    for (mut camera_transform, mut player_camera) in camera_query.iter_mut() {
        for event in mouse_wheel_events.iter() {
            player_camera.distance -= event.y;
            player_camera.distance = player_camera.distance.clamp(player_camera.min_distance, player_camera.max_distance);
        }

        if mouse_input.pressed(MouseButton::Right) || camera_type.0 == FreeCamState::ThirdPersonLocked {
            for event in mouse_events.iter() {
                let sens = config.sens;
                player_camera.yaw_radians -= event.delta.x * sens;
                player_camera.pitch_radians -= event.delta.y * sens;
                player_camera.pitch_radians = player_camera.pitch_radians.clamp(0.05 * PI, 0.95 * PI);
            }
        }

        let camera_location =
            Quat::from_rotation_y(player_camera.yaw_radians)
            * Quat::from_rotation_z(player_camera.pitch_radians)
            * Vec3::Y
            * player_camera.distance
            + player_transform.translation;

        let new_transform = Transform::from_translation(camera_location)
            .looking_at(player_transform.translation + 0.5 * Vec3::Y, Vec3::Y);

        camera_transform.translation = new_transform.translation;
        camera_transform.rotation = new_transform.rotation;
    }
}

pub fn camera_topdown_system(
    mut camera_query: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    let player_transform = match player_query.get_single() {
        Ok(s) => s,
        Err(_) => return,
    };
    for (mut cam_transform, player_camera) in camera_query.iter_mut() {
        
        // Stay above player
        let new_transform = Transform::from_translation(
            player_transform.translation + Vec3::new(0., player_camera.distance, 0.))
            .looking_at(player_transform.translation, Vec3::Z
        );
        cam_transform.translation = new_transform.translation;
        // cam_transform.rotation = new_transform.rotation; //Quat::from_rotation_x(-PI/2.)// + Quat::from_rotation_(player_camera.yaw_radians);
        let look_direction = Vec3::new(0.0, -1.0, 0.0);
        let up_direction = Vec3::new(0.0, 0.0, 1.0); // assumes z-axis is up
        let rotation_quat = Quat::from_rotation_y(player_camera.yaw_radians);
        dbg!(rotation_quat);
        let rotated_look_direction = rotation_quat * look_direction;
        cam_transform.rotation = Quat::from_rotation_arc(rotated_look_direction, up_direction);
        cam_transform.translation = Quat::from_rotation_y(player_camera.yaw_radians)
            * Quat::from_rotation_x(player_camera.pitch_radians)
            * Vec3::Y
            * player_camera.distance
            + player_transform.translation;

    }
}

pub fn camera_zoom_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut PlayerCamera>,
){
    let mut camera_follow = match camera_query.get_single_mut() {
        Ok(s) => s,
        Err(_) => return,
    };
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
}

