use std::time::Duration;

use crate::states::GameState;
use bevy::{prelude::*, time::common_conditions::on_timer};
use shared::{
    event::{client::{WorldData, PlayerDisconnected, PlayerConnected}, server::{ConnectRequest, Heartbeat}, ERFE},
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
            (shared::event::client::drain_events, receive_world_data, on_connect, on_disconnect)
                .run_if(in_state(GameState::ClientConnected)),
        )
        .add_systems(
            Update,
            send_heartbeat
                .run_if(on_timer(Duration::from_millis(200)))
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

fn receive_world_data(
    mut world_data: ERFE<WorldData>,
    mut commands: Commands,
) {
    for event in world_data.read() {
        info!(?event, "Server has returned world data!");
        let my_name = &event.event.your_name;
        let my_id = event.event.your_id;
        for other_player in &event.event.players {
            info!(?other_player);
        }
    }
}

fn send_heartbeat(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
) {
    let event = EventToServer::Heartbeat(Heartbeat {});
    send_event_to_server(&sr.handler, mse.0, &event);
}

fn on_disconnect(
    mut dc_info: ERFE<PlayerDisconnected>
) {
    for event in dc_info.read() {
        info!(?event);
    }
}


fn on_connect(
    mut c_info: ERFE<PlayerConnected>
) {
    for event in c_info.read() {
        info!(?event);
    }
}
