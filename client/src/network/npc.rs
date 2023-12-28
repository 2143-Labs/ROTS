use bevy::prelude::*;
use shared::{
    event::{client::SpawnUnit, ERFE},
    AnyUnit,
};

use crate::{
    network::{build_healthbar, OtherPlayer},
    player::{MovementIntention, PlayerName},
    states::GameState,
};

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
    mut pd: ERFE<SpawnUnit>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //parent: Query<Entity, With<ChatContainer>>,
    //players: Query<(&NetEntId, &PlayerName), With<AnyPlayer>>,
    //mut er: EventReader<Chat>,
    //time: Res<Time>,
) {
    for event in pd.read() {
        let ud = &event.event.data;
        match ud.unit {
            shared::event::UnitType::Player { name } => {
                let cube = SceneBundle {
                    scene: asset_server.load("tadpole.gltf#Scene0"),
                    transform: ud.transform,
                    ..default()
                };

                commands
                    .spawn((
                        cube,
                        OtherPlayer,
                        PlayerName(event.data.name.clone()),
                        MovementIntention(Vec2::ZERO),
                        Name::new(format!("Player: {}", event.data.name)),
                        // their NetEntId is a component
                        ud.ent_id,
                        ud.health,
                        AnyUnit,
                    ))
                    .with_children(|s| build_healthbar(s, &mut meshes, &mut materials));
            }
            shared::event::UnitType::NPC { npc_type } => {
                let npc = &event.event;
                let cube = SceneBundle {
                    scene: asset_server.load(npc_type.model()),
                    transform: Transform::from_translation(ud.transform.location),
                    ..default()
                };

                commands
                    .spawn((
                        cube,
                        ud.ent_id,
                        ud.health,
                        npc_type.clone(),
                        Name::new(format!("NPC: {:?}", npc_type)),
                        MovementIntention(Vec2::ZERO),
                        AnyUnit,
                    ))
                    .with_children(|s| build_healthbar(s, &mut meshes, &mut materials));
            }
        }
    }
}
