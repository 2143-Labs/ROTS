use bevy::prelude::*;

use crate::states::{CameraState, GameState};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_system(setup_camera.run_if(in_state(GameState::Ready).and_then(run_once())))
}

#[derive(Component)]
struct PlayerCamera; // tag entity to make it always face the camera

#[derive(Reflect, Component)]
pub struct CameraFollow {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub dragging: bool,
    pub degrees: f32,
    pub old_degrees: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            dragging: false,
            degrees: 0.,
            old_degrees: 0.,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
enum ProjectionType {
    #[default]
    Perspective,
    Orthographic,
}

fn swap_camera(camera : &mut Camera3d,  orthographic: ProjectionType) {
    // Create a new camera entity and add a transform component to it
    // let mut camera_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 10.0));
    // camera_transform.look_at(Vec3::default(), Vec3::Y);
    let proj= match orthographic {
        ProjectionType::Perspective => Projection::Perspective( PerspectiveProjection {
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            // aspect_ratio: 1.0,
            ..Default::default()
        }),
        ProjectionType::Orthographic => Projection::Orthographic( OrthographicProjection {
            near: 0.0,
            far: 100.0,
            ..Default::default()
        }),
    };
    Camera3d.projection = proj;
    // Add the camera component to the entity
}

pub fn camera_topdown_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<Input<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<Camera3d>>,
    mut player_query: Query<(&Transform, &mut Player), Without<CameraFollow>>,
    keyboard_input: Res<Input<KeyCode>>,
){
    if let Ok((player_transform, mut player)) = player_query.get_single_mut() {
    }
}


pub fn camera_pan_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<Input<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<Camera3d>>,
    mut player_query: Query<(&Transform, &mut Player), Without<CameraFollow>>,
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

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            RaycastSource::<MyRaycastSet>::new_transform_empty(),
        ))
        .insert(CameraFollow::default())
        .insert(Name::new("Camera"))
        .insert(PlayerCamera);
}
