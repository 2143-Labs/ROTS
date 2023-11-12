use bevy::prelude::*;
use crate::{states::GameState, despawn_all_component, player::spawn_player_sprite};
use self::systems::{menu_select, check_is_next_to_button};

mod scene;
mod systems;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuItem;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), (scene::spawn_menu_scene, spawn_player_sprite))
            .add_systems(Update, (menu_select, check_is_next_to_button).distributive_run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), despawn_all_component::<MenuItem>)
            ;
    }
}
