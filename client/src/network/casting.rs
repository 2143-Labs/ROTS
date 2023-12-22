use std::time::Duration;

use bevy::prelude::*;
use shared::event::{ERFE, client::SomeoneCast, NetEntId, spells::ShootingData};

use crate::states::GameState;

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    update_casts,
                )
            )
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

#[derive(Component)]
pub struct DespawnTime(pub Timer);

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


fn update_casts(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Transform, &ShootingData, &mut DespawnTime)>,
    time: Res<Time<Virtual>>,
) {
    for (bullet_ent, mut bullet_tfm, shot_data, mut despawn_timer) in &mut bullets {
        if despawn_timer.0.tick(time.delta()).finished() {
            commands.entity(bullet_ent).despawn_recursive();
            return;
        }

        let offset = (shot_data.target - shot_data.shot_from) * despawn_timer.0.elapsed_secs();
        let new_bullet_loc = shot_data.shot_from + offset;
        bullet_tfm.translation = new_bullet_loc;
    }
}
