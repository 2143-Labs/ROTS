use std::ops::DerefMut;

use bevy::prelude::*;
use message_io::network::{Transport, NetEvent};
use shared::{GameNetEvent, ServerResources, event::PlayerConnect};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_networking_server)
            .insert_resource(ServerResources::default())
            .add_event::<shared::event::PlayerConnect>()
            .add_system(tick_server);
    }
}

fn setup_networking_server(
    event_list_res: Res<ServerResources>,
) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let (server, _) = match handler.network().connect(Transport::Udp, "127.0.0.1:3042") {
        Ok(d) => d,
        Err(_) => return error!("failed to connect to active server",),
    };

    info!("probably connected");

    let connect_event = GameNetEvent::PlayerConnect(shared::event::PlayerConnect {
        name: "2143".into(),
    });
    let event_json = serde_json::to_string(&connect_event).unwrap();
    dbg!(&event_json);
    handler.network().send(server, event_json.as_bytes());
    info!("sent json");

    let res_copy = event_list_res.clone();

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //let s = from_utf8(data);
                //info!(?s);
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });
}

fn tick_server(
    event_list_res: Res<ServerResources>,
    mut ev_player_connect: EventWriter<PlayerConnect>,
) {
    let events_to_process = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for event in events_to_process {
        let (_endpoint, e) = event;
        match e {
            GameNetEvent::Noop => warn!("Got noop event"),
            GameNetEvent::PlayerConnect(p) => ev_player_connect.send(p),
            GameNetEvent::PlayerList(p_list) => ev_player_connect.send_batch(p_list),
            _ => {}
        }
    }
}

fn on_player_connect(
    mut ev_player_connect: EventReader<PlayerConnect>,
) {
    for e in &mut ev_player_connect {
        info!("Got a player connection event {e:?}");
    }
}

