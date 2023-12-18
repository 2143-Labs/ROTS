use serde::{Serialize, de::DeserializeOwned};

//use crate::{despawn_all_component, player::spawn_player_sprite, states::GameState};
use bevy::prelude::*;

use shared::events::{NEC2S, ClientRequests};
use shared::{Config, EventList, ServerNodeHandler};
use shared::events::{self, connect::ClientConnect};

use message_io::network::{Endpoint, NetEvent};

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
pub struct ServerEndpoints {
    pub main: Endpoint,
}

#[derive(States, Clone, Hash, PartialEq, Eq, Debug, Default)]
pub enum NetworkingState {
    #[default]
    NotConnected,
    Connecting,
    Connected,
}

fn to_server_event_sender<T>(
    mut events: EventReader<T>,
    server_handlers: Res<ServerNodeHandler>,
    server_endpoints: Res<ServerEndpoints>,
) where T: Event + Serialize {
    for _e in events.read() {
        let event_json = serde_json::to_string(_e).unwrap();
        server_handlers.handler.network().send(server_endpoints.main, event_json.as_bytes());
        // serialize to json and write to socket
    }
}

fn from_server_event_receiver<T>(
    mut events: EventWriter<T>,
    //
) where T: Event + DeserializeOwned {
    let json = "";
    match serde_json::from_str(json) {
        Ok(data) => events.send(data),
        Err(_) => {}, // TODO
    }
}

pub fn init_msg_to_server<N: NEC2S>(
    app: &mut App,
){
    app.add_event::<N::ClientSend>();
    app.add_event::<N::ServerResponse>();

    app.add_systems(Update, to_server_event_sender::<N::ClientSend>.run_if(in_state(NetworkingState::Connecting)));
    app.add_systems(Update, from_server_event_receiver::<N::ServerResponse>.run_if(in_state(NetworkingState::Connecting)));

    // add to event checking loop
}

fn go_connect(
    config: Res<Config>,
    mut networking_state: ResMut<NextState<NetworkingState>>,
    mut commands: Commands,
) {
    info!("Trying to connect to server...");
    let (handler, listener) = message_io::node::split::<()>();

    let con_str = (&*config.ip, config.port);

    let (server, _) = handler
        .network()
        .connect(message_io::network::Transport::Tcp, con_str)
        .expect("Failed to connect ot server");

    info!("probably connected");

    let server_event_list = EventList::<shared::events::ServerResponses> {
        event_list: Default::default(),
    };

    let server_handler = ServerNodeHandler {
        handler: handler.clone(),
    };

    let server_event_list_clone = server_event_list.clone();

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {}
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //server_event_list_clone.event_list.lock().unwrap().push((endpoint, from_utf8(data).unwrap().to_owned()));
                let event = serde_json::from_slice(data).unwrap();
                server_event_list_clone.event_list.lock().unwrap().push((endpoint, event));
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
    mut client_connect: EventWriter<ClientRequests>,
) {
    let connect_event = events::connect::Req {
        name: config.name.clone(),
    };

    client_connect.send(ClientRequests::Connect(connect_event));

    info!("sent json");
}

#[derive(Resource, Clone, Debug)]
pub struct WorldInitInfo {
    pub our_name: String,
    pub client_id: u64,
}

fn wait_request_packet(
    mut client_connect: EventReader<events::connect::Res>,
) {
    for e in client_connect.read() {
        let world_info = WorldInitInfo {
            our_name: e.your_name.clone(),
            client_id: e.client_id,
        };

        info!("World Info: {:?}", world_info);
    }
}
