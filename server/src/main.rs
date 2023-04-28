use std::ops::DerefMut;

use rand::{thread_rng, Rng};
use shared::{EventToServer, EventToClient, NetEntId};
use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::{Duration, HashMap}, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent, Endpoint}};
use shared::{event::PlayerInfo, ServerResources, EventFromEndpoint};

fn main() {
    info!("Main Start");
    let mut app = App::new();

    let res = start_server();

    app
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 120.0)))
        .insert_resource(res)
        .insert_resource(EndpointToNetId::default())
        .add_event::<EventFromEndpoint<PlayerInfo>>()
        .add_event::<(NetEntId, Transform)>()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_system(on_player_connect)
        .add_system(tick_net_server);

    app.run();
}

fn start_server() -> ServerResources<EventToServer> {
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
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

    res
}

#[derive(Resource, Default)]
struct EndpointToNetId {
    map: HashMap<Endpoint, NetEntId>,
}

fn tick_net_server(
    event_list_res: Res<ServerResources<EventToServer>>,
    entity_mapping: Res<EndpointToNetId>,
    mut ev_player_connect: EventWriter<EventFromEndpoint<PlayerInfo>>,
    mut ev_player_movement: EventWriter<(NetEntId, Transform)>,
) {
    let events_to_process: Vec<_> = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());

    for (_endpoint, e) in events_to_process {
        match e {
            EventToServer::Noop => todo!(),
            EventToServer::Connect { name } => {
                let id = NetEntId(thread_rng().gen());
                let ev = PlayerInfo {
                    name,
                    id,
                };

                ev_player_connect.send(EventFromEndpoint::new(_endpoint, ev));
            },
            EventToServer::UpdatePos(new_pos) => {
                match entity_mapping.map.get(&_endpoint) {
                    Some(id) => ev_player_movement.send((*id, new_pos)),
                    None => error!("Failed to match endpoint {_endpoint:?}to id"),
                }
            },
            _ => todo!(),
        }
    }
}

#[derive(Component)]
struct GameNetClient {
    name: String,
    endpoint: Endpoint,
}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerInfo>>,
    other_players: Query<(&GameNetClient, &NetEntId)>,
    mut commands: Commands,
    event_list_res: Res<ServerResources<EventToServer>>,
) {
    for e in &mut ev_player_connect {
        info!("Got a player connection event {e:?}");
        let new_client = GameNetClient {
            endpoint: e.endpoint,
            name: e.event.name.clone(),
        };


        // First, notify all existing players about the new player
        // Also collect all their names to use later
        let connect_event = EventToClient::PlayerConnect(e.event.clone());
        let mut names = vec![];
        for (player, ent_id) in &other_players {
            let data = serde_json::to_string(&connect_event).unwrap();
            event_list_res.handler
                .network()
                .send(player.endpoint, data.as_bytes());

            names.push(PlayerInfo {
                name: player.name.clone(),
                id: *ent_id,
            });
        }

        // Next, tell the new player about the existing players
        let connect_event = EventToClient::PlayerList(names);
        let data = serde_json::to_string(&connect_event).unwrap();
        event_list_res.handler
            .network()
            .send(e.endpoint, data.as_bytes());

        // Finally, add our client to the ECS
        commands
            .spawn_empty()
            .insert(new_client)
            .insert(e.event.id)
            ;
    }
}
