use std::fs::OpenOptions;

use bevy::{prelude::*, utils::HashMap};

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EventFromEndpoint<RunChatCommand>>()
            .add_event::<SaveSaveState>()
            .add_event::<LoadSaveState>()
            .add_systems(
                Update,
                (
                    on_chat,
                    on_chat_command,
                    on_save_savestate,
                    on_load_savestate,
                )
                    .run_if(in_state(ServerState::Running)),
            );
    }
}

use clap::{Args, Parser};
use message_io::{network::Endpoint, node::NodeHandler};
use serde::{Deserialize, Serialize};
use shared::{
    event::{
        client::{Chat, SpawnUnit, UnitDie},
        server::SendChat,
        spells::NPC,
        EventFromEndpoint, NetEntId, UnitData, UnitType, ERFE,
    },
    netlib::{send_event_to_server, EventToClient, EventToServer, ServerResources},
    stats::Health,
    AnyUnit,
};

use crate::{
    game_manager::GameManagerState, ConnectedPlayerName, EndpointToNetId, PlayerEndpoint,
    ServerState,
};
#[derive(Parser, Debug, Event)]
#[command(name = "chat_command")]
#[command(bin_name = "/")]
pub enum ChatCommand {
    Spawn(CmdSpawnUnit),
    List(CmdListUnits),
    ///SaveState
    S,
    ///StateState Load
    L,

    Start,
    Stop,
}

/// Spawn a unit
#[derive(Args, Debug)]
pub struct CmdSpawnUnit {
    pub enemy_type: NPC,
}

/// List all the units on the server
#[derive(Args, Debug)]
pub struct CmdListUnits {
    #[arg(short, default_value = "true")]
    verbose: bool,
}

#[derive(Event)]
struct RunChatCommand {
    runner: NetEntId,
    command: ChatCommand,
}

fn on_chat(
    mut pd: ERFE<SendChat>,
    endpoint_mapping: Res<EndpointToNetId>,
    clients: Query<&PlayerEndpoint, With<AnyUnit>>,
    sr: Res<ServerResources<EventToServer>>,
    mut cmd: EventWriter<EventFromEndpoint<RunChatCommand>>,
) {
    for chat in pd.read() {
        if let Some(chatter_net_id) = endpoint_mapping.map.get(&chat.endpoint) {
            let text = &chat.event.text;
            info!(?chatter_net_id, text, "Chat");
            if text.starts_with('/') {
                // if it starts with /, its a command parse it using clap
                let cmd_parts = text.split_at(1).1.split(' ');
                // we have to add an extra argment at the start so that clap parses it correctly
                let cmd_parts = [""].iter().cloned().chain(cmd_parts);
                match ChatCommand::try_parse_from(cmd_parts) {
                    Ok(x) => {
                        let event = EventToClient::Chat(Chat {
                            source: None,
                            text: format!("Running {:?}", x),
                        });
                        send_event_to_server(&sr.handler, chat.endpoint, &event);

                        // Trigger event to send the chat command
                        cmd.send(EventFromEndpoint {
                            endpoint: chat.endpoint,
                            event: RunChatCommand {
                                runner: *chatter_net_id,
                                command: x,
                            },
                        });
                    }
                    Err(k) => {
                        let event = EventToClient::Chat(Chat {
                            source: None,
                            text: format!("Error in {}\n{}", text, k),
                        });
                        send_event_to_server(&sr.handler, chat.endpoint, &event);
                    }
                };
            } else {
                let event = EventToClient::Chat(Chat {
                    source: Some(*chatter_net_id),
                    text: text.clone(),
                });

                for c_net_client in &clients {
                    send_event_to_server(&sr.handler, c_net_client.0, &event);
                }
            }
        }
    }
}

#[derive(Event, Clone, Debug)]
struct LoadSaveState(SaveStateData);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveStateData {
    npcs: Vec<SaveStateUnit>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveStateUnit {
    hp: Health,
    unit: UnitType,
    transform: Transform,
}

type NPCSavestateQuery<'a> = (&'a NPC, &'a Health, &'a Transform);
impl From<NPCSavestateQuery<'_>> for SaveStateUnit {
    fn from(value: NPCSavestateQuery<'_>) -> Self {
        Self {
            hp: *value.1,
            unit: UnitType::NPC {
                npc_type: value.0.clone(),
            },
            transform: *value.2,
        }
    }
}

/// This represents a sendable connection to a client
#[derive(Clone)]
struct NetworkableClientEndpoint {
    endpoint: Endpoint,
    handler: NodeHandler<()>,
}

#[derive(Event, Clone)]
struct SaveSaveState(NetworkableClientEndpoint);

fn on_save_savestate(
    mut cmd: EventReader<SaveSaveState>,
    cur_npc_query: Query<NPCSavestateQuery<'_>, With<AnyUnit>>,
    // TODO capture players?
) {
    for save_state_request in cmd.read() {
        let npcs: Vec<_> = cur_npc_query
            .iter()
            .map(SaveStateUnit::from)
            .collect();

        let all_data = SaveStateData { npcs };

        let location = "./save.savestate.json";
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(location)
            .unwrap();
        serde_json::to_writer_pretty(file, &all_data).unwrap();

        let event = EventToClient::Chat(Chat {
            source: None,
            text: format!("Saved state: {}", location,),
        });
        send_event_to_server(
            &save_state_request.0.handler,
            save_state_request.0.endpoint,
            &event,
        );
    }
}

fn on_load_savestate(
    mut cmd: EventReader<LoadSaveState>,
    cur_npc_query: Query<(&NetEntId, &NPC), With<AnyUnit>>,
    mut spawn_npc: EventWriter<SpawnUnit>,
    mut unit_die: EventWriter<UnitDie>,
) {
    for save_data in cmd.read() {
        // Remove all current entities
        unit_die.send_batch(cur_npc_query.iter().map(|(net_ent_id, _)| UnitDie {
            id: *net_ent_id,
            disappear: true,
        }));

        for unit in save_data.0.npcs.clone() {
            spawn_npc.send(SpawnUnit {
                data: UnitData {
                    unit: unit.unit,
                    health: unit.hp,
                    transform: unit.transform,
                    ent_id: NetEntId(rand::random()),
                },
            });
        }
    }
}

fn on_chat_command(
    mut cmd: EventReader<EventFromEndpoint<RunChatCommand>>,
    players: Query<(Entity, &Transform, &NetEntId, &ConnectedPlayerName)>,
    list_npc_query: Query<(&NetEntId, &NPC), With<AnyUnit>>,
    list_player_query: Query<(&NetEntId, &ConnectedPlayerName), With<AnyUnit>>,
    sr: Res<ServerResources<EventToServer>>,
    mut spawn_npc: EventWriter<SpawnUnit>,
    mut load_savestate: EventWriter<LoadSaveState>,
    mut save_savestate: EventWriter<SaveSaveState>,
    mut next_game_manager_state: ResMut<NextState<GameManagerState>>,
    cur_game_manager_state: Res<State<GameManagerState>>,
) {
    for command in cmd.read() {
        let (_runner_ent, runner_tfm, _runner_net_ent, _runner_name) = match players
            .iter()
            .find(|(_, _, &id, _)| id == command.event.runner)
        {
            Some(s) => s,
            None => continue,
        };

        match &command.event.command {
            ChatCommand::Spawn(unit) => {
                spawn_npc.send(SpawnUnit {
                    data: UnitData {
                        unit: shared::event::UnitType::NPC {
                            npc_type: unit.enemy_type.clone(),
                        },
                        ent_id: NetEntId(rand::random()),
                        health: unit.enemy_type.get_base_health(),
                        transform: Transform::from_translation(
                            runner_tfm.translation * Vec3::new(1., 0., 1.),
                        ),
                    },
                });
            }
            ChatCommand::List(_) => {
                let mut enemies = HashMap::new();
                let mut player_names = vec![];

                for (_neid, npc) in &list_npc_query {
                    *enemies.entry(npc).or_insert(0) += 1;
                }
                for (_neid, ConnectedPlayerName { name }) in &list_player_query {
                    player_names.push(name);
                }

                let event = EventToClient::Chat(Chat {
                    source: None,
                    text: format!(
                        "{cur_game_manager_state:?}: Players: {:?} || NPCs: {:?}",
                        player_names, enemies
                    ),
                });
                send_event_to_server(&sr.handler, command.endpoint, &event);
            }
            ChatCommand::Start => {
                info!("gaming");
                next_game_manager_state.set(GameManagerState::Playing);
            }
            ChatCommand::Stop => {
                next_game_manager_state.set(GameManagerState::NotPlaying);
            }
            ChatCommand::S => {
                info!("Saving State");
                save_savestate.send(SaveSaveState(NetworkableClientEndpoint {
                    endpoint: command.endpoint,
                    handler: sr.handler.clone(),
                }));
            }
            ChatCommand::L => {
                let location = "./save.savestate.json";
                let file = OpenOptions::new().read(true).open(location).unwrap();
                let data: SaveStateData = serde_json::from_reader(file).unwrap();
                load_savestate.send(LoadSaveState(data));

                let event = EventToClient::Chat(Chat {
                    source: None,
                    text: format!("Loaded savestate: {}", location,),
                });
                send_event_to_server(&sr.handler, command.endpoint, &event);
            }
        }
    }
}
