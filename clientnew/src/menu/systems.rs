use bevy::{prelude::*, ecs::query::QuerySingleError};

use crate::{states::GameState, player::Player};

use super::scene::{SelectedButton, MenuButton};

pub fn menu_select(
    keyboard_input: Res<Input<KeyCode>>,
    _config: Res<shared::Config>,
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Query<&MenuButton, With<SelectedButton>>,
) {

    if !keyboard_input.just_pressed(KeyCode::H) {
        return;
    }

    let button = match buttons.get_single() {
        Ok(button) => button,
        Err(QuerySingleError::NoEntities(_)) => {
            // Play sound error
            info!("Not near a button");
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            info!("Somehow got multiple selected buttons?");
            return;
        }
    };

    info!("Clicked button {button:?}");
    match button {
        MenuButton::Connect => {
            game_state.set(GameState::Connecting);
        },
        MenuButton::CreateServer => {
            game_state.set(GameState::CreateServer);
        },
        MenuButton::Quit => {
            game_state.set(GameState::Quit);
        }
    }
}

pub fn check_is_next_to_button(
    mut commands: Commands,
    players: Query<(Entity, &Player, &Transform)>,
    mut buttons: Query<(Entity, &MenuButton, &mut Transform, Option<&SelectedButton>), Without<Player>>,
    time: Res<Time>
) {
    // This system will add or remove the `SelectedButton` component, and make the buttons spin
    let max_dist = 2.0;
    for (_player_ent, _player, player_transform) in &players {
        for (button_ent, button_type, mut button_transform, selected) in &mut buttons {
            if selected.is_some() {
                button_transform.rotate_x(time.delta_seconds() * 3.0);
                button_transform.rotate_y(time.delta_seconds() * 1.0);
                button_transform.rotate_z(time.delta_seconds() * 1.0);
                let dist = player_transform.translation.distance(button_transform.translation);
                if dist > max_dist {
                    info!("Left range of {button_type:?}");
                    commands.entity(button_ent).remove::<SelectedButton>();
                }
            } else {
                button_transform.rotation = Quat::from_rotation_y(0.0);
                let dist = player_transform.translation.distance(button_transform.translation);
                if dist < max_dist {
                    info!("Approached {button_type:?}");
                    commands.entity(button_ent).insert(SelectedButton);
                }
            }
        }
    }
}

