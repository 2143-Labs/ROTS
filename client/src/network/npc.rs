use bevy::prelude::*;
use shared::event::{client::NewNPC, ERFE};

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
                (on_npc_spawn).run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn on_npc_spawn(
    mut pd: ERFE<NewNPC>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //parent: Query<Entity, With<ChatContainer>>,
    //players: Query<(&NetEntId, &PlayerName), With<AnyPlayer>>,
    //mut er: EventReader<Chat>,
    //time: Res<Time>,
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
