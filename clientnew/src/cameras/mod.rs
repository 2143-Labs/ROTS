mod thirdperson;

use bevy::{
    prelude::*, window::CursorGrabMode,
};

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
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_state::<FreeCamState>()
            .add_systems(Update, toggle_camera_mode)
            .add_systems(
                Update,
                (thirdperson::player_movement_thirdperson, thirdperson::wow_camera_system)
                    .distributive_run_if(in_state(FreeCamState::ThirdPersonLocked)),
            )
            .add_systems(
                Update,
                (thirdperson::player_movement_thirdperson, thirdperson::wow_camera_system, thirdperson::q_e_rotate_cam)
                    .distributive_run_if(in_state(FreeCamState::ThirdPersonFreeMouse)),
            )
            .register_type::<Jumper>()
        ;
    }
}

#[derive(Component)]
struct PrimaryCamera; // tag entity to make it always face the camera

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        thirdperson::CameraFollow::default(),
        Name::new("Camera"),
        PrimaryCamera,
    ));
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum FreeCamState {
    #[default]
    ThirdPersonLocked,
    ThirdPersonFreeMouse,
    Free,
}

pub fn toggle_camera_mode(
    cam_state: Res<State<FreeCamState>>,
    mut next_state: ResMut<NextState<FreeCamState>>,
    input: Res<Input<KeyCode>>,
    mut windows_query: Query<&mut Window>,
) {
    if input.just_pressed(KeyCode::X) {
        if let Ok(mut window) = windows_query.get_single_mut() {
            window.cursor.grab_mode = match window.cursor.grab_mode {
                CursorGrabMode::None => CursorGrabMode::Locked,
                CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
            };
            window.cursor.visible = !window.cursor.visible;
        };
    }
    if input.just_pressed(KeyCode::C) {
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
