use bevy::prelude::*;
use shared::{netlib::{MainServerEndpoint, setup_client, EventToClient, ServerResources, send_event_to_server, EventToServer}, Config, event::WorldData};
use crate::states::GameState;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::ClientConnecting), (
                // Setup the client and immediatly advance the state
                setup_client::<EventToClient>,
                |mut state: ResMut<NextState<GameState>>| {
                    state.set(GameState::ClientConnected)
                }
            ))
            .add_systems(OnEnter(GameState::ClientConnected), (send_connect_packet,))

            // TODO
            .add_systems(Update, (
                shared::netlib::drain_events::<EventToClient>,
            ).run_if(resource_exists::<MainServerEndpoint>()));
    }
}


fn send_connect_packet(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
) {
    //TODO how to eliminate clone here when pulling from config?
    let event = EventToServer::Connect { name: config.name.clone() };
    send_event_to_server(&sr.handler, mse.0, &event);
    //let event = EventToServer::UpdatePos(Transform::from_xyz(0.0, 1.0, 2.0));
    //send_event_to_server(&sr.handler, mse.0, &event);
}

fn receive_world_data(
    mut world_data: EventReader<WorldData>,
) {
    for event in world_data.read() {
        info!("Server has returned world data!");
    }
}
