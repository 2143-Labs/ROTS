use crate::states::GameState;
use bevy::prelude::*;
use shared::{
    event::{client::WorldData, server::ConnectRequest, ERFE},
    netlib::{
        send_event_to_server, setup_client, EventToClient, EventToServer, MainServerEndpoint,
        ServerResources,
    },
    Config,
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        shared::event::client::register_events(app);
        app.add_systems(
            OnEnter(GameState::ClientConnecting),
            (
                // Setup the client and immediatly advance the state
                setup_client::<EventToClient>,
                |mut state: ResMut<NextState<GameState>>| state.set(GameState::ClientConnected),
            ),
        )
        .add_systems(OnEnter(GameState::ClientConnected), (send_connect_packet,))
        .add_systems(
            Update,
            (shared::event::client::drain_events, receive_world_data)
                .run_if(in_state(GameState::ClientConnected)),
        );
    }
}

fn send_connect_packet(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
) {
    let event = EventToServer::ConnectRequest(ConnectRequest {
        name: config.name.clone(),
    });
    send_event_to_server(&sr.handler, mse.0, &event);
    info!("Sent connection packet to {}", mse.0);
}

fn receive_world_data(mut world_data: ERFE<WorldData>) {
    for event in world_data.read() {
        info!(?event, "Server has returned world data!");
    }
}
