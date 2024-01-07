use bevy::prelude::*;
use shared::{
    event::{
        client::{SpawnUnit, UnitDie},
        NetEntId, ERFE,
    },
    unit::MovementIntention,
    AnyUnit,
};

use crate::{
    network::{build_healthbar, OtherPlayer},
    player::{PlayerName, PrimaryUnitControl},
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
                (on_npc_spawn, on_unit_die).run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn on_npc_spawn(
    mut pd: EventReader<SpawnUnit>,
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
        let ud = &event.data;
        match &ud.unit {
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
                        PlayerName(name.clone()),
                        MovementIntention(Vec2::ZERO),
                        Name::new(format!("Player: {name}")),
                        // their NetEntId is a component
                        ud.ent_id,
                        ud.health,
                        AnyUnit,
                    ))
                    .with_children(|s| build_healthbar(s, &mut meshes, &mut materials, Vec3::ZERO));
            }
            shared::event::UnitType::NPC { npc_type } => {
                //commands.insert_resource(Animation(asset_server.load(npc_type.animation())));
                let cube = SceneBundle {
                    scene: asset_server.load(npc_type.model()),
                    transform: Transform::from_translation(ud.transform.translation),
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
                    .with_children(|s| {
                        build_healthbar(s, &mut meshes, &mut materials, Vec3::new(0.0, 5.0, 0.0))
                    });
            }
        }
    }
}

fn on_unit_die(
    mut er: ERFE<UnitDie>,
    units: Query<(Entity, &NetEntId, &Transform, Has<PrimaryUnitControl>), With<AnyUnit>>,
    mut commands: Commands,
) {
    for e in er.read() {
        for (unit_ent, &unit_ent_id, tfm, is_local_player) in &units {
            if e.event.id == unit_ent_id {
                commands.entity(unit_ent).despawn_recursive();

                if is_local_player {
                    // Spawn a spectator cameras
                    commands.spawn((
                        PrimaryUnitControl,
                        TransformBundle::from_transform(Transform::from_translation(
                            tfm.translation,
                        )),
                    ));
                }
            }
        }
    }
}
