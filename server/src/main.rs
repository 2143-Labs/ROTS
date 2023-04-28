use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent, Endpoint}};
use shared::{event::PlayerConnect, ServerResources, EventFromEndpoint, GameNetEvent};

fn main() {
    info!("Main Start");
    let mut app = App::new();

    let res = start_server();

    app
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 120.0)))
        .insert_resource(res)
        .add_event::<EventFromEndpoint<PlayerConnect>>()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_system(on_player_connect)
        .add_system(shared::tick_server);

    app.run();
}

fn start_server() -> ServerResources {
    warn!("Start Server");

    let (handler, listener) = node::split::<()>();

    let res = ServerResources {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let res_copy = res.clone();

    std::thread::spawn(move || {
        handler.network().listen(Transport::Udp, "0.0.0.0:3042").unwrap();

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

    res
}

#[derive(Component)]
struct GameNetClient {
    name: String,
    endpoint: Endpoint,
}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerConnect>>,
    other_players: Query<&GameNetClient>,
    mut commands: Commands,
    event_list_res: Res<ServerResources>,
) {
    for e in &mut ev_player_connect {
        info!("Got a player connection event {e:?}");
        let new_client = GameNetClient {
            endpoint: e.endpoint,
            name: e.event.name.clone(),
        };

        // First, notify all existing players about the new player
        // Also collect all their names to use later
        let connect_event = GameNetEvent::PlayerConnect(PlayerConnect { name: e.event.name.clone() });
        let mut names = vec![];
        for player in &other_players {
            let data = serde_json::to_string(&connect_event).unwrap();
            event_list_res.handler
                .network()
                .send(player.endpoint, data.as_bytes());

            names.push(&*player.name);
        }

        // Next, tell them about the existing players
        let connect_event = GameNetEvent::PlayerList(
            names
                .into_iter()
                .map(|name| PlayerConnect {
                    name: name.to_string()
                })
                .collect()
        );
        let data = serde_json::to_string(&connect_event).unwrap();
        event_list_res.handler
            .network()
            .send(e.endpoint, data.as_bytes());

        // Finally, add our client to the ECS
        commands
            .spawn_empty()
            .insert(new_client);
    }
}
