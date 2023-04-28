use bevy::prelude::*;
use message_io::network::{NetEvent, Transport};
use rand::{thread_rng, Rng};
use shared::{event::PlayerConnect, GameNetEvent, ServerResources, EventFromEndpoint};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_networking_server)
            .insert_resource(ServerResources::default())
            .add_event::<EventFromEndpoint<PlayerConnect>>()
            .add_system(on_player_connect)
            .add_system(shared::tick_server);
    }
}

fn setup_networking_server(event_list_res: Res<ServerResources>) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let (server, _) = match handler.network().connect(Transport::Udp, "127.0.0.1:3042") {
        Ok(d) => d,
        Err(_) => return error!("failed to connect to active server",),
    };

    info!("probably connected");

    let name = thread_rng().gen_range(1..10000);

    let connect_event = GameNetEvent::PlayerConnect(shared::event::PlayerConnect {
        name: format!("Player #{name}"),
    });
    let event_json = serde_json::to_string(&connect_event).unwrap();
    dbg!(&event_json);
    handler.network().send(server, event_json.as_bytes());
    info!("sent json");

    let res_copy = event_list_res.clone();

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
}

fn on_player_connect(mut ev_player_connect: EventReader<EventFromEndpoint<PlayerConnect>>) {
    for e in &mut ev_player_connect {
        info!("TODO spawn player in world... {e:?}");
    }
}
