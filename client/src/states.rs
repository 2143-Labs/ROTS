use bevy::prelude::*;

#[derive(States, Reflect, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,

    /// See also [shared::netlib::NetworkConnectionTarget]
    ClientConnecting,
    ClientSendRequestPacket,
    ClientConnected,

    Quit,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Quit), quit_event);
    }
}

fn quit_event(mut app_exit_events: ResMut<Events<bevy::app::AppExit>>) {
    app_exit_events.send(bevy::app::AppExit);
}
