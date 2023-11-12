use bevy::prelude::*;

#[derive(States, Reflect, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
    }
}
