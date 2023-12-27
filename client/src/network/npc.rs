use std::{fs::OpenOptions, io::Read, os::unix::ffi::OsStrExt};

use bevy::prelude::*;
use shared::{event::{client::NewNPC, ERFE, server::Spray}, netlib::{EventToServer, ServerResources, MainServerEndpoint, EventToClient, send_event_to_server}};

use crate::states::GameState;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_state::<ChatState>()
            //.add_event::<Chat>()
            //.add_systems(Startup, setup_panel)
            //.add_systems(Update, on_chat_toggle.run_if(shared::GameAction::Chat.just_pressed()))
            .add_systems(
                Update,
                (on_npc_spawn, file_drop).run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn on_npc_spawn(
    mut pd: ERFE<NewNPC>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in pd.read() {
        let npc = &event.event;
        let cube = SceneBundle {
            scene: asset_server.load("penguin.gltf#Scene0"),
            transform: Transform::from_translation(npc.spawn_commands.location),
            ..default()
        };
        info!(?npc);

        commands.spawn((cube, npc.id, npc.spawn_commands.npc.clone()));
        //.with_children(|s| build_healthbar(s, &mut meshes, &mut materials));
    }
}

fn file_drop(
    mut dnd_evr: EventReader<FileDragAndDrop>,
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for ev in dnd_evr.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = ev {
            info!("Dropped a file");
            let mut fo = match OpenOptions::new().read(true).open(path_buf) {
                Ok(s) => s,
                Err(e) => {
                    error!(?e);
                    continue;
                }
            };

            let mut data = Vec::new();
            fo.read_to_end(&mut data).unwrap();

            info!("Sending spray data to server!");
            let event = EventToServer::Spray(Spray {
                filename: String::from_utf8_lossy(path_buf.as_os_str().as_bytes()).to_string(),
                data,
            });
            send_event_to_server(&sr.handler, mse.0, &event);
        }
    }
}
