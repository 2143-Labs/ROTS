use std::str::from_utf8;
use std::sync::{Arc, Mutex};

//use crate::{despawn_all_component, player::spawn_player_sprite, states::GameState};
use bevy::prelude::*;

use message_io::node::NodeHandler;
use shared::Config;
use shared::events::NEC2S;
use shared::events::connect::ConnectEventClient;
use shared::events::{init_msg_to_server, connect::ClientConnect};

use message_io::network::{Endpoint, NetEvent, Transport};

use crate::states::GameState;

pub struct NetworkingPlugin;

//#[derive(Component)]
//pub struct NetworkingItem;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connecting), go_connect);
        app.add_state::<NetworkingState>();

        init_msg_to_server::<ClientConnect>(app);
        app.add_systems(OnEnter(NetworkingState::Connecting), send_connect_packet);
        app.add_systems(Update, wait_request_packet.run_if(in_state(NetworkingState::Connecting)));

    }
}

#[derive(Resource)]
struct ServerEndpoints {
    pub main: Endpoint,
}

#[derive(Resource)]
struct ServerHandler {
    pub handler: NodeHandler<()>,
}

#[derive(Resource, Clone)]
struct ServerEventList {
    pub event_list: Arc<Mutex<Vec<(Endpoint, String)>>>,
}

#[derive(States, Clone, Hash, PartialEq, Eq, Debug, Default)]
enum NetworkingState {
    #[default]
    NotConnected,
    Connecting,
    Connected,
}

fn go_connect(
    config: Res<Config>,
    mut networking_state: ResMut<NextState<NetworkingState>>,
    mut commands: Commands,
) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let con_str = (&*config.ip, config.port);

    let (server, _) = handler
        .network()
        .connect(message_io::network::Transport::Udp, con_str)
        .expect("Failed to connect ot server");

    info!("probably connected");

    let server_event_list = ServerEventList {
        event_list: Default::default(),
    };

    let server_handler = ServerHandler {
        handler: handler.clone(),
    };

    let server_event_list_clone = server_event_list.clone();

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {}
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                server_event_list_clone.event_list.lock().unwrap().push((endpoint, from_utf8(data).unwrap().to_owned()));
                //let event = serde_json::from_slice(data).unwrap();
                //server_event_list_clone.event_list.lock().unwrap().push((endpoint, event));
            }
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

    commands.insert_resource(server_event_list);
    commands.insert_resource(server_handler);
    commands.insert_resource(ServerEndpoints {
        main: server,
    });
    networking_state.set(NetworkingState::Connecting);
}

fn send_connect_packet(
    config: Res<Config>,
    mut client_connect: EventWriter<ConnectEventClient>,
) {
    let connect_event = shared::events::connect::ConnectEventClient {
        name: config.name.clone(),
    };

    client_connect.send(connect_event);

    info!("sent json");
}


fn wait_request_packet(
    mut client_connect: EventReader<shared::events::connect::ConnectEventResp>,
) {
    for e in client_connect.read() {
        info!("{:?}", e);
    }
}
