//use crate::{despawn_all_component, player::spawn_player_sprite, states::GameState};
use bevy::prelude::*;
use message_io::network::NetEvent;
use shared::{Config, EventToServer};

use crate::states::GameState;

pub struct NetworkingPlugin;

//#[derive(Component)]
//pub struct NetworkingItem;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connecting), go_connect);
        app.add_systems(OnEnter(GameState::Connecting), go_start);
    }
}

fn go_connect(
    config: Res<Config>,
    //mut networking_state: ResMut<NextState<NetworkingState>>,
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

    //let res = ServerResources::<EventToClient> {
        //handler: handler.clone(),
        //event_list: Default::default(),
    //};

    //let res_copy = res.clone();

    //let mse = MainServerEndpoint(server.clone());

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {}
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //let s = from_utf8(data);
                //info!(?s);
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            }
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

    //let name = match &config.name {
        //Some(name) => name.clone(),
        //None => {
            //let random_id = thread_rng().gen_range(1..10000);
            //format!("Player #{random_id}")
        //}
    //};
    let connect_event = shared::events::connect::ConnectEventClient {
        name: config.name,
    };

    //let connect_event = EventToServer::Connect { name: config.name };
    let event_json = serde_json::to_string(&connect_event).unwrap();
    handler.network().send(server, event_json.as_bytes());

    info!("sent json");

    commands.insert_resource(res);
    commands.insert_resource(mse);
    networking_state.set(NetworkingState::WaitingForServer);
}
