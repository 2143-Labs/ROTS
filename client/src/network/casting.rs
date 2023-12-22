use bevy::prelude::*;
use shared::event::{ERFE, client::SomeoneCast, NetEntId};

use crate::states::GameState;

use super::OtherPlayer;

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    on_someone_cast,
                )
                    .run_if(in_state(GameState::ClientConnected)),
            )

            ;
    }
}

fn on_someone_cast(
    mut someone_cast: ERFE<SomeoneCast>,
    other_players: Query<
        (Entity, &NetEntId, &Transform),
    >,
    mut commands: Commands,
    //TODO dont actually spawn a cube on cast
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for cast in someone_cast.read() {
        for (_ply_ent, ply_net_ent, ply_tfm) in &other_players {

            if &cast.event.id == ply_net_ent {
                let cube = PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                    material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                    transform: Transform::from_translation(ply_tfm.translation),
                    ..Default::default()
                };

                commands.spawn((
                    cube,
                ));
            }
        }
    }
}

