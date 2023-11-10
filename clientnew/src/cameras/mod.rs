use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*, window::CursorGrabMode,
};
use shared::Config;

#[derive(Reflect, Component)]
pub struct Jumper {
    //pub cooldown: f32,
    pub timer: Timer,
}

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    };

    commands.spawn((
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
        for event in mouse_wheel_events.read() {
            camera_follow.distance -= event.y;
            camera_follow.distance = camera_follow
                .distance
                .clamp(camera_follow.min_distance, camera_follow.max_distance);
        }

        if mouse_input.pressed(MouseButton::Right)
            || *camera_type == FreeCamState::ThirdPersonLocked
        {
            for event in mouse_events.read() {
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


pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_state::<FreeCamState>()
            .add_systems(Update, toggle_freecam)
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
        ;
    }
}

#[derive(Component)]
struct PlayerCamera; // tag entity to make it always face the camera

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

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow::default(),
        Name::new("Camera"),
        PlayerCamera,
    ));
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum FreeCamState {
    #[default]
    ThirdPersonLocked,
    ThirdPersonFreeMouse,
    Free,
}

pub fn toggle_freecam(
    mut players: Query<Entity, With<CameraFollow>>,
    mut commands: Commands,
    cam_state: Res<State<FreeCamState>>,
    mut next_state: ResMut<NextState<FreeCamState>>,
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
        next_state.set(match **cam_state {
            FreeCamState::Free => {
                //for player in players.iter_mut() {
                    //commands.entity(player).remove::<FlyCamera>();
                //}
                FreeCamState::ThirdPersonLocked
            }
            FreeCamState::ThirdPersonLocked => {
                FreeCamState::ThirdPersonFreeMouse
            }
            FreeCamState::ThirdPersonFreeMouse => {
                //for player in players.iter_mut() {
                    //commands.entity(player).insert(FlyCamera::default());
                //}
                FreeCamState::Free
            }
        });
    }
}
