use bevy::prelude::*;

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RunChatCommand>()
            .add_systems(Update, (on_chat, on_chat_command).run_if(in_state(ServerState::Running)));

    }
}

use clap::{Parser, Args};
use shared::{event::{ERFE, server::SendChat, client::Chat, NetEntId, spells::{NPC, SpawnNPC}}, AnyPlayer, netlib::{ServerResources, EventToServer, EventToClient, send_event_to_server}};

use crate::{ServerState, EndpointToNetId, PlayerEndpoint, ConnectedPlayerName};
#[derive(Parser, Debug, Event)]
#[command(name = "chat_command")]
#[command(bin_name = "/")]
pub enum ChatCommand {
    Spawn(SpawnEnemy),
}

#[derive(Args, Debug)]
pub struct SpawnEnemy {
    pub enemy_type: NPC,
}

#[derive(Event)]
struct RunChatCommand {
    runner: NetEntId,
    command: ChatCommand,
}

fn on_chat(
    mut pd: ERFE<SendChat>,
    endpoint_mapping: Res<EndpointToNetId>,
    clients: Query<&PlayerEndpoint, With<AnyPlayer>>,
    sr: Res<ServerResources<EventToServer>>,
    mut cmd: EventWriter<RunChatCommand>,
) {

    for chat in pd.read() {
        if let Some(moved_net_id) = endpoint_mapping.map.get(&chat.endpoint) {
            let text = &chat.event.text;
            if text.starts_with("/") {
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
                        cmd.send(RunChatCommand {
                            runner: *moved_net_id,
                            command: x
                        });
                    },
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
                    source: Some(*moved_net_id),
                    text: text.clone(),
                });


                for c_net_client in &clients {
                    send_event_to_server(&sr.handler, c_net_client.0, &event);
                }
            }
        }
    }
}

fn on_chat_command(
    mut cmd: EventReader<RunChatCommand>,
    players: Query<(Entity, &Transform, &NetEntId, &ConnectedPlayerName)>,
    mut spawn_npc: EventWriter<SpawnNPC>,
) {
    for command in cmd.read() {
        let (_runner_ent, runner_tfm, _runner_net_ent, runner_name) = match players.iter().find(|(_, _, &id, _)| id == command.runner) {
            Some(s) => s,
            None => continue,
        };

        match &command.command {
            ChatCommand::Spawn(se) => {
                spawn_npc.send(SpawnNPC {
                    location: runner_tfm.translation,
                    npc: NPC::Penguin,
                });
            },
        }
    }
}
