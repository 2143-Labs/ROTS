use bevy::{prelude::*, window::CursorGrabMode};
use bevy_fly_camera::FlyCamera;

use crate::{setup::CameraFollow};

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]

pub enum FreeCamState {
    Free,
    #[default]
    Locked,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    Ready,
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
    println!("::: ESCAPE PRESSED! :::");
    next_state.set(match cam_state.0 {
        FreeCamState::Free => {
            println!("::: FreeCamState::Free :::");
            for player in players.iter_mut() {
                commands.entity(player).remove::<FlyCamera>();
            }
            FreeCamState::Locked
        }
        FreeCamState::Locked => {
            println!("::: FreeCamState::Locked :::");
            for player in players.iter_mut() {
                commands.entity(player).insert(FlyCamera::default());
            }
            FreeCamState::Free
        }
    });
}
}
