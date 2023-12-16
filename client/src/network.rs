use bevy::prelude::*;
use message_io::{network::{Transport, Endpoint}, node::NodeHandler};
use shared::{Config, EventToClient, ServerResources, EventToServer};

use crate::{player::Player, cameras::notifications::Notification, states::GameState};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::ClientConnecting), setup_server)
            ;
    }
}

pub fn send_event_to_server(handler: &NodeHandler<()>, endpoint: Endpoint, event: &EventToServer) {
    handler.network().send(endpoint, &postcard::to_stdvec(&event).unwrap());
}

fn setup_server(
    mut commands: Commands,
    config: Res<shared::Config>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    info!("Seting up the server!");

    let (handler, listener) = message_io::node::split::<()>();

    let res = ServerResources::<EventToClient> {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let con_str = (&*config.ip, config.port);
    let (endpoint, _addr) = handler.network().connect(Transport::Udp, con_str).unwrap();

    commands.insert_resource(res.clone());

    let event = EventToServer::Connect { name: config.name.clone() };
    send_event_to_server(&handler, endpoint, &event);

    //std::thread::spawn(move || {
        //listener.for_each(|event| on_node_event(&res, event));
    //});

    game_state.set(GameState::ClientConnectWaitServer);
}
