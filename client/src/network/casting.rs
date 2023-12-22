use std::time::Duration;

use bevy::prelude::*;
use shared::{event::{ERFE, client::SomeoneCast, NetEntId, spells::ShootingData}, casting::{DespawnTime, SharedCastingPlugin}};

use crate::states::GameState;

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SharedCastingPlugin)
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
                match cast.event.cast {
                    shared::event::server::Cast::Teleport(target) => {
                        info!(?target, "Someone teleported")
                    },
                    shared::event::server::Cast::Shoot(ref dat) => {
                        let cube = PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                            material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                            transform: Transform::from_translation(ply_tfm.translation),
                            ..Default::default()
                        };

                        commands.spawn((
                            cube,
                            dat.clone(),
                            DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                            // TODO Add a netentid for referencing this item later
                        ));
                    },
                }
            }
        }
    }
}
