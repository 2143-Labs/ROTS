use std::{
    collections::HashMap,
    sync::{atomic::AtomicI16, Arc},
    time::Duration,
};

use bevy::{log::LogPlugin, prelude::*};
use bevy_time::common_conditions::on_timer;
use message_io::network::Endpoint;
use rand::Rng;
use shared::{
    event::{
        client::{PlayerDisconnected, SomeoneMoved, SpawnUnit, WorldData},
        server::{ChangeMovement, Heartbeat},
        spells::NPC,
        NetEntId, UnitData, UnitType, ERFE,
    },
    netlib::{
        send_event_to_server, EventToClient, EventToServer, NetworkConnectionTarget,
        ServerResources,
    },
    stats::Health,
    Config, ConfigPlugin,
};

/// How often to run the system
const HEARTBEAT_MILLIS: u64 = 200;
/// How long until disconnect
const HEARTBEAT_TIMEOUT: u64 = 1000;
/// How long do you have to connect, as a multipler of the heartbeart timeout.
/// If the timeout is 1000 ms, then `5` would mean you have `5000ms` to connect.
const HEARTBEAT_CONNECTION_GRACE_PERIOD: u64 = 5;

#[derive(States, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum ServerState {
    #[default]
    NotReady,
    Starting,
    Running,
}

#[derive(Resource, Default)]
struct HeartbeatList {
    heartbeats: HashMap<NetEntId, Arc<AtomicI16>>,
}

#[derive(Resource, Default)]
struct EndpointToNetId {
    map: HashMap<Endpoint, NetEntId>,
}

#[derive(Debug, Component)]
struct ConnectedPlayerName {
    pub name: String,
}

#[derive(Debug, Component)]
struct PlayerEndpoint(Endpoint);

#[derive(Event)]
struct PlayerDisconnect {
    ent: NetEntId,
}

pub mod casting_spells;
pub mod chat;
pub mod npc;

fn main() {
    info!("Main Start");
    let mut app = App::new();

    shared::event::server::register_events(&mut app);
    app.insert_resource(EndpointToNetId::default())
        .insert_resource(HeartbeatList::default())
        .add_event::<PlayerDisconnect>()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins((
            ConfigPlugin,
            casting_spells::CastingPlugin,
            chat::ChatPlugin,
            npc::NPCPlugin,
            //StatusPlugin,
        ))
        .add_state::<ServerState>()
        .add_systems(
            Startup,
            (
                add_network_connection_info_from_config,
                |mut state: ResMut<NextState<ServerState>>| state.set(ServerState::Starting),
            ),
        )
        .add_systems(
            OnEnter(ServerState::Starting),
            (
                shared::netlib::setup_server::<EventToServer>,
                |mut state: ResMut<NextState<ServerState>>| state.set(ServerState::Running),
            ),
        )
        .add_systems(
            Update,
            (
                shared::event::server::drain_events,
                on_player_connect,
                on_player_disconnect,
                on_player_heartbeat,
                on_movement,
            )
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(
            Update,
            check_heartbeats.run_if(on_timer(Duration::from_millis(200))),
        );

    app.run();
}

fn add_network_connection_info_from_config(config: Res<Config>, mut commands: Commands) {
    commands.insert_resource(NetworkConnectionTarget {
        ip: config.ip.clone(),
        port: config.port,
    });
}

fn on_player_connect(
    mut new_players: ERFE<shared::event::server::ConnectRequest>,
    mut heartbeat_mapping: ResMut<HeartbeatList>,
    mut endpoint_to_net_id: ResMut<EndpointToNetId>,
    clients: Query<(
        &Transform,
        &PlayerEndpoint,
        &NetEntId,
        &ConnectedPlayerName,
        &Health,
    )>,
    npcs: Query<(&Transform, &NetEntId, &Health, &NPC)>,
    sr: Res<ServerResources<EventToServer>>,
    _config: Res<Config>,
    mut commands: Commands,
) {
    for player in new_players.read() {
        info!(?player);

        // Generate their name
        let name = player
            .event
            .name
            .clone()
            .unwrap_or_else(|| format!("Player #{}", rand::thread_rng().gen_range(1..10000)));

        //if they are too far, just put them at the spawn
        let default_spawn = player
            .event
            .my_location
            .with_translation(Vec3::new(0.0, 0.0, 0.0));
        let spawn_location = if player
            .event
            .my_location
            .translation
            .distance_squared(default_spawn.translation)
            > 10.0
        {
            default_spawn
        } else {
            player.event.my_location
        };

        let new_player_data = UnitData {
            ent_id: NetEntId::random(),
            health: Health::default(),
            transform: spawn_location,
            unit: UnitType::Player { name: name.clone() },
        };

        let event = EventToClient::SpawnUnit(SpawnUnit {
            data: new_player_data.clone(),
        });

        // The new player we just spawned is the first unit in the list that we send to the client.
        let mut unit_list = vec![new_player_data.clone()];

        for (c_tfm, c_net_client, &ent_id, ConnectedPlayerName { name: c_name }, &health) in
            &clients
        {
            unit_list.push(UnitData {
                unit: UnitType::Player {
                    name: c_name.clone(),
                },
                ent_id,
                health,
                transform: *c_tfm,
            });

            // Tell all other clients,
            // also notify their player data to send
            send_event_to_server(&sr.handler, c_net_client.0, &event);
        }

        for (&transform, &ent_id, &health, npc_type) in &npcs {
            unit_list.push(UnitData {
                unit: UnitType::NPC {
                    npc_type: npc_type.clone(),
                },
                ent_id,
                health,
                transform,
            });
        }

        // Directly spawn the unit here, instead of sending a SpawnUnit event.
        commands.spawn((
            ConnectedPlayerName { name },
            new_player_data.ent_id,
            new_player_data.health,
            new_player_data.transform,
            PlayerEndpoint(player.endpoint),
            // Transform component used for generic systems
            shared::AnyUnit,
        ));

        // Each time we miss a heartbeat, we increment the Atomic counter.
        // So, we initially set this to negative number to give extra time for the initial
        // connection.
        let hb_grace_period =
            (HEARTBEAT_CONNECTION_GRACE_PERIOD - 1) * (HEARTBEAT_TIMEOUT / HEARTBEAT_MILLIS);

        heartbeat_mapping.heartbeats.insert(
            new_player_data.ent_id,
            Arc::new(AtomicI16::new(-(hb_grace_period as i16))),
        );

        endpoint_to_net_id
            .map
            .insert(player.endpoint, new_player_data.ent_id);

        // Finally, tell the client all this info.
        let event = EventToClient::WorldData(WorldData {
            your_unit_id: new_player_data.ent_id,
            unit_data: unit_list,
        });
        send_event_to_server(&sr.handler, player.endpoint, &event);
    }
}

fn check_heartbeats(
    heartbeat_mapping: Res<HeartbeatList>,
    mut on_disconnect: EventWriter<PlayerDisconnect>,
) {
    for (ent_id, beats_missed) in &heartbeat_mapping.heartbeats {
        let beats = beats_missed.fetch_add(1, std::sync::atomic::Ordering::Acquire);
        if beats >= (HEARTBEAT_TIMEOUT / HEARTBEAT_MILLIS) as i16 {
            warn!("Missed {beats} beats, disconnecting {ent_id:?}");
            on_disconnect.send(PlayerDisconnect { ent: *ent_id });
        }
    }
}

fn on_player_disconnect(
    mut pd: EventReader<PlayerDisconnect>,
    clients: Query<(Entity, &PlayerEndpoint, &NetEntId), With<ConnectedPlayerName>>,
    mut commands: Commands,
    mut heartbeat_mapping: ResMut<HeartbeatList>,
    sr: Res<ServerResources<EventToServer>>,
) {
    for player in pd.read() {
        heartbeat_mapping.heartbeats.remove(&player.ent);

        let event = EventToClient::PlayerDisconnected(PlayerDisconnected { id: player.ent });
        for (_c_ent, c_net_client, _c_net_ent) in &clients {
            send_event_to_server(&sr.handler, c_net_client.0, &event);
            if _c_net_ent == &player.ent {
                commands.entity(_c_ent).despawn_recursive();
            }
        }
    }
}

fn on_player_heartbeat(
    mut pd: ERFE<Heartbeat>,
    heartbeat_mapping: Res<HeartbeatList>,
    endpoint_mapping: Res<EndpointToNetId>,
) {
    for hb in pd.read() {
        // TODO tryblocks?
        if let Some(id) = endpoint_mapping.map.get(&hb.endpoint) {
            if let Some(hb) = heartbeat_mapping.heartbeats.get(id) {
                hb.fetch_min(0, std::sync::atomic::Ordering::Release);
            }
        }
    }
}

fn on_movement(
    mut pd: ERFE<ChangeMovement>,
    endpoint_mapping: Res<EndpointToNetId>,
    mut clients: Query<(&PlayerEndpoint, &NetEntId, &mut Transform), With<ConnectedPlayerName>>,
    sr: Res<ServerResources<EventToServer>>,
) {
    for movement in pd.read() {
        if let Some(moved_net_id) = endpoint_mapping.map.get(&movement.endpoint) {
            let event = EventToClient::SomeoneMoved(SomeoneMoved {
                id: *moved_net_id,
                movement: movement.event.clone(),
            });

            for (c_net_client, c_net_ent, mut c_tfm) in &mut clients {
                if moved_net_id == c_net_ent {
                    // If this person moved, update their transform serverside
                    match movement.event {
                        ChangeMovement::SetTransform(new_tfm) => *c_tfm = new_tfm,
                        _ => {}
                    }
                } else {
                    // Else, just rebroadcast the packet to everyone else
                    send_event_to_server(&sr.handler, c_net_client.0, &event);
                }
            }
        }
    }
}
