use std::time::Duration;

use bevy::prelude::*;
use shared::{
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin},
    event::{
        client::{BulletHit, SomeoneCast},
        NetEntId, ERFE,
    },
    AnyPlayer,
};

use crate::{
    cameras::notifications::Notification,
    player::{Player, PlayerName},
    states::GameState,
};

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedCastingPlugin)
            .add_event::<WeTeleported>()
            .add_systems(
                Update,
                (on_someone_cast, on_someone_hit, on_us_tp)
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

#[derive(Event)]
struct WeTeleported(Vec3);

fn on_us_tp(
    mut local_player: Query<&mut Transform, With<Player>>,
    mut ev_r: EventReader<WeTeleported>,
) {
    for ev in ev_r.read() {
        local_player.single_mut().translation = ev.0;
    }
}

fn on_someone_cast(
    mut someone_cast: ERFE<SomeoneCast>,
    other_players: Query<(Entity, &NetEntId, &Transform, Has<Player>), With<AnyPlayer>>,
    mut commands: Commands,
    //TODO dont actually spawn a cube on cast
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_w: EventWriter<WeTeleported>,
    asset_server: Res<AssetServer>,
) {
    for cast in someone_cast.read() {
        for (_ply_ent, ply_net_ent, ply_tfm, is_us) in &other_players {
            if &cast.event.caster_id == ply_net_ent {
                match cast.event.cast {
                    shared::event::server::Cast::Teleport(target) => {
                        // Spawn a sound at both the source and dest
                        // TODO only play both if you go a far enough distance
                        for loc in &[target /* ply_tfm.translation */] {
                            commands.spawn((
                                TransformBundle::from_transform(Transform::from_translation(*loc)),
                                //Transform::from_xyz(0.0, 0.0, 0.0),
                                AudioBundle {
                                    source: asset_server.load("sounds/teleport.ogg"),
                                    settings: PlaybackSettings::DESPAWN.with_spatial(true),
                                    ..default()
                                },
                            ));
                        }

                        match is_us {
                            true => {
                                ev_w.send(WeTeleported(target));
                            }
                            false => info!("Someone else teleported"),
                        }
                    }
                    shared::event::server::Cast::Shoot(ref dat) => {
                        let cube = PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                            material: materials.add(Color::rgb(0.0, 0.3, 0.7).into()),
                            transform: Transform::from_translation(ply_tfm.translation),
                            ..Default::default()
                        };

                        commands.spawn((
                            cube,
                            dat.clone(),
                            cast.event.cast_id,
                            CasterNetId(cast.event.caster_id),
                            DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                            // TODO Add a netentid for referencing this item later
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn on_someone_hit(
    mut someone_hit: ERFE<BulletHit>,
    all_plys: Query<(&NetEntId, &Transform, &PlayerName, Has<Player>), With<AnyPlayer>>,
    mut notifs: EventWriter<Notification>,
    bullets: Query<(Entity, &NetEntId, &CasterNetId)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for hit in someone_hit.read() {
        let mut bullet_caster_id = None;
        for (_bullet_ent, bullet_ent_id, attacker_net_id) in &bullets {
            if bullet_ent_id == &hit.event.bullet {
                bullet_caster_id = Some(attacker_net_id);
            }
        }

        // if we dont know about the bullet, return
        let bullet_caster_id = match bullet_caster_id {
            Some(s) => s.0,
            None => return warn!("Unknown bullet"),
        };

        let mut attacker_name = None;
        let mut defender_name = None;

        for (ply_id, ply_tfm, PlayerName(name), is_us) in &all_plys {
            if ply_id == &hit.event.player {
                defender_name = Some(name);
                if is_us {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sounds/hit.ogg"),
                        ..default()
                    });
                    info!("We got hit!");
                } else {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sounds/hitmarker.ogg"),
                        ..default()
                    });
                }
            }

            if ply_id == &bullet_caster_id {
                attacker_name = Some(name);
            }
        }

        match (attacker_name, defender_name) {
            (Some(atk), Some(def)) => {
                info!(?atk, ?def, "Hit!");
                notifs.send(Notification(format!("{atk} hit {def}")));
            }
            (Some(atk), None) => {
                warn!(?atk, "Unknown defender");
            }
            (None, Some(def)) => {
                warn!(?def, "Unknown attacker");
            }
            (None, None) => {
                warn!("Unknown bullet");
            }
        }
    }
}
