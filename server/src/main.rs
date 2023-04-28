use std::ops::DerefMut;

use rand::{thread_rng, Rng};
use shared::{EventToServer, EventToClient, NetEntId, BulletPhysics};
use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::{Duration, HashMap}, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent, Endpoint}};
use shared::{event::PlayerInfo, ServerResources, EventFromEndpoint};

fn main() {
    info!("Main Start");
    let mut app = App::new();

    let res = start_server();

    app
        .add_plugin(LogPlugin::default())
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 120.0)))
        .insert_resource(res)
        .insert_resource(EndpointToNetId::default())
        .add_event::<EventFromEndpoint<PlayerInfo>>()
        .add_event::<(NetEntId, Transform)>()
        .add_event::<(NetEntId, BulletPhysics)>()
        .add_plugins(MinimalPlugins)
        .add_system(on_player_connect)
        .add_system(tick_net_server)
        .add_system(send_shooting_to_all_players)
        .add_system(send_movement_to_all_players);

    app.run();
}

fn start_server() -> ServerResources<EventToServer> {
    let (handler, listener) = node::split::<()>();

    let res = ServerResources {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let res_copy = res.clone();

    std::thread::spawn(move || {
        handler.network().listen(Transport::Udp, "0.0.0.0:3000").unwrap();

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
    mut entity_mapping: ResMut<EndpointToNetId>,
    mut ev_player_connect: EventWriter<EventFromEndpoint<PlayerInfo>>,
    mut ev_player_movement: EventWriter<(NetEntId, Transform)>,
    mut ev_player_shooting: EventWriter<(NetEntId, BulletPhysics)>,
) {
    let events_to_process: Vec<_> = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());

    for (_endpoint, e) in events_to_process {
        match e {
            EventToServer::Noop => todo!(),
            EventToServer::Connect { name } => {
                info!("A player has connected with name '{name}'");
                let id = NetEntId(thread_rng().gen());
                let ev = PlayerInfo {
                    name,
                    id,
                };

                ev_player_connect.send(EventFromEndpoint::new(_endpoint, ev));

                entity_mapping.map.insert(_endpoint, id);
            },
            EventToServer::UpdatePos(new_pos) => {
                match entity_mapping.map.get(&_endpoint) {
                    Some(id) => {
                        ev_player_movement.send((*id, new_pos));
                    }
                    None => error!("Failed to match endpoint {_endpoint:?}to id"),
                }
            },
            EventToServer::ShootBullet(phys) => {
                match entity_mapping.map.get(&_endpoint) {
                    Some(id) => {
                        info!("Player {id:?} is shooting");
                        ev_player_shooting.send((*id, phys));
                    }
                    None => error!("Failed to match endpoint {_endpoint:?}to id"),
                }
            }
            _ => todo!(),
        }
    }
}

fn send_movement_to_all_players(
    mut ev_player_movement: EventReader<(NetEntId, Transform)>,
    event_list_res: Res<ServerResources<EventToServer>>,
    players: Query<&GameNetClient>,
) {
    let events: Vec<_> = ev_player_movement
        .iter()
        .map(|x| EventToClient::UpdatePos(x.0, x.1))
        .collect();

    for client in &players {
        for event in &events {
            let events_as_str = serde_json::to_string(&event).unwrap();
            event_list_res.handler
                .network()
                .send(client.endpoint, events_as_str.as_bytes());
        }
    }
}

fn send_shooting_to_all_players(
    mut ev_shoot: EventReader<(NetEntId, BulletPhysics)>,
    event_list_res: Res<ServerResources<EventToServer>>,
    players: Query<&GameNetClient>,
) {
    let events: Vec<_> = ev_shoot
        .iter()
        .map(|x| EventToClient::ShootBullet(x.0, x.1.clone()))
        .collect();

    for client in &players {
        for event in &events {
            let events_as_str = serde_json::to_string(&event).unwrap();
            event_list_res.handler
                .network()
                .send(client.endpoint, events_as_str.as_bytes());
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
            .insert(e.event.id);
    }
}
