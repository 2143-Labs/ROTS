use bevy::prelude::*;
use shared::{netlib::{MainServerEndpoint, setup_client, EventToClient, ServerResources, send_event_to_server, EventToServer}, Config, event::{server::ConnectRequest, ERFE}};
use crate::states::GameState;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        shared::event::client::register_events(app);
        app
            .add_systems(OnEnter(GameState::ClientConnecting), (
                // Setup the client and immediatly advance the state
                setup_client::<EventToClient>,
                |mut state: ResMut<NextState<GameState>>| {
                    state.set(GameState::ClientConnected)
                }
            ))
            .add_systems(OnEnter(GameState::ClientConnected), (
                send_connect_packet,
            ))
            .add_systems(Update, (
                shared::event::client::drain_events,
                receive_world_data,
            ).run_if(in_state(GameState::ClientConnected)));
    }
}


fn send_connect_packet(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
) {
    //TODO how to eliminate clone here when pulling from config?
    let event = EventToServer::ConnectRequest(ConnectRequest {name: config.name.clone() });
    send_event_to_server(&sr.handler, mse.0, &event);
    info!("Sent connection packet to {}", mse.0);
    //let event = EventToServer::UpdatePos(Transform::from_xyz(0.0, 1.0, 2.0));
    //send_event_to_server(&sr.handler, mse.0, &event);
}

fn receive_world_data(
    mut world_data: ERFE<shared::event::client::WorldData>,
) {
    for event in world_data.read() {
        info!(?event, "Server has returned world data!");
    }
}
