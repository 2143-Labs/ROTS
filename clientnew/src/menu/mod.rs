use bevy::prelude::*;

use crate::{states::GameState, despawn_all_component};

use self::scene::menu_select;

mod scene;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), (scene::spawn_menu_scene, crate::cameras::spawn_player_sprite))
            .add_systems(Update, (menu_select).distributive_run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), despawn_all_component::<scene::MenuItem>)
            ;
    }
}
