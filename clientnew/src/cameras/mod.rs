mod thirdperson;

use bevy::{prelude::*, window::CursorGrabMode};

#[derive(Component)]
pub struct FaceCamera; // tag entity to make it always face the camera

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_state::<FreeCamState>()
            .add_systems(Update, toggle_camera_mode)
            .add_systems(
                Update,
                (
                    thirdperson::player_movement_thirdperson,
                    thirdperson::wow_camera_system,
                )
                    .distributive_run_if(in_state(FreeCamState::ThirdPersonLocked)),
            )
            .add_systems(
                Update,
                (
                    thirdperson::player_movement_thirdperson,
                    thirdperson::wow_camera_system,
                    thirdperson::q_e_rotate_cam,
                )
                    .distributive_run_if(in_state(FreeCamState::ThirdPersonFreeMouse)),
            );
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
            FreeCamState::ThirdPersonLocked => FreeCamState::ThirdPersonFreeMouse,
            FreeCamState::ThirdPersonFreeMouse => {
                //for player in players.iter_mut() {
                //commands.entity(player).insert(FlyCamera::default());
                //}
                FreeCamState::Free
            }
        });
    }
}
