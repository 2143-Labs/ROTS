use bevy::prelude::*;
use message_io::network::{NetEvent, Transport};
use rand::{thread_rng, Rng};
use shared::{event::PlayerConnect, GameNetEvent, ServerResources, EventFromEndpoint};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let server = setup_networking_server();
        app
            .insert_resource(server)
            .add_event::<EventFromEndpoint<PlayerConnect>>()
            .add_system(on_player_connect)
            .add_system(shared::tick_server);
    }
}

fn setup_networking_server() -> ServerResources {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let (server, _) = handler.network().connect(Transport::Udp, "127.0.0.1:3042").expect("Failed to connect ot server");

    info!("probably connected");

    let name = thread_rng().gen_range(1..10000);

    let connect_event = GameNetEvent::PlayerConnect(shared::event::PlayerConnect {
        name: format!("Player #{name}"),
    });
    let event_json = serde_json::to_string(&connect_event).unwrap();
    handler.network().send(server, event_json.as_bytes());
    info!("sent json");

    let res = ServerResources {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let res_copy = res.clone();


    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {},
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

    res
}

fn on_player_connect(mut ev_player_connect: EventReader<EventFromEndpoint<PlayerConnect>>) {
    for e in &mut ev_player_connect {
        info!("TODO spawn player in world... {e:?}");
    }
}
