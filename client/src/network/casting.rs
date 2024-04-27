use std::time::Duration;

use bevy::prelude::*;
use shared::{
    animations::{AnimationTimer, CastNetId, CastPointTimer, DoCast},
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin, TargetedBullet},
    event::{
        client::{BulletHit, SomeoneCast},
        NetEntId, ERFE,
    },
    AnyUnit,
};

use crate::{
    player::{Player, PlayerName},
    states::GameState,
};

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedCastingPlugin)
            .add_event::<WeTeleported>()
            .add_event::<DoCast>()
            .add_systems(
                Update,
                (on_someone_cast, on_someone_hit, on_us_tp, do_cast_finish)
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

//#[derive(Component)]
//struct ShootTargetProj;

fn do_cast_finish(
    mut do_cast: EventReader<DoCast>,
    mut commands: Commands,
    //mut units: Query<(&NetEntId, &mut Transform, ), With<AnyUnit>>,
    local_player_ent_id: Query<&NetEntId, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut ev_w: EventWriter<WeTeleported>,
) {
    for DoCast(cast) in do_cast.read() {
        debug!(?cast, "Cast has completed");
        //let mut maybe_caster = None;
        //for (unit_ent, unit_tfm) {
        //if
        //}
        match cast.cast {
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

                match local_player_ent_id.single() == &cast.caster_id {
                    true => {
                        ev_w.send(WeTeleported(target));
                    }
                    false => trace!("Someone else teleported"),
                }
            }
            shared::event::server::Cast::Shoot(ref dat) => {
                let cube = PbrBundle {
                    mesh: meshes.add(Mesh::from(Cuboid { half_size: Vec3::splat(0.15) })),
                    material: materials.add(Color::rgb(0.0, 0.3, 0.7)),
                    transform: Transform::from_translation(dat.shot_from),
                    ..Default::default()
                };

                trace!(?cast.cast_id, "Spawning a bullet with id");
                commands.spawn((
                    cube,
                    dat.clone(),
                    cast.cast_id,
                    CasterNetId(cast.caster_id),
                    DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                ));
            }
            shared::event::server::Cast::ShootTargeted(from_loc, ref net_id) => {
                let cube = PbrBundle {
                    mesh: meshes.add(Mesh::from(Cuboid { half_size: Vec3::splat(0.25) })),
                    material: materials.add(Color::rgb(0.0, 0.3, 0.7)),
                    transform: Transform::from_translation(from_loc),
                    ..Default::default()
                };

                commands.spawn((
                    cube,
                    cast.cast_id,
                    TargetedBullet(from_loc, *net_id),
                    CasterNetId(cast.caster_id),
                    // TODO hardcoded proj duration
                    DespawnTime(Timer::new(Duration::from_secs(1), TimerMode::Once)),
                ));
            }
            ref rest => {
                trace!(?rest, "Cast event");
            }
        }
    }
}

/// Conains the cast id of the tp
#[derive(Component)]
pub struct TpCube(pub NetEntId);

fn on_someone_cast(
    mut someone_cast: ERFE<SomeoneCast>,
    other_players: Query<(Entity, &NetEntId, &Transform, Has<Player>), With<AnyUnit>>,
    mut commands: Commands,
    //TODO dont actually spawn a cube on cast
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //mut ev_w: EventWriter<WeTeleported>,
    //asset_server: Res<AssetServer>,
) {
    for cast in someone_cast.read() {
        for (casting_ent, net_ent_id, _caster_tfm, is_us) in &other_players {
            if &cast.event.caster_id == net_ent_id {
                let cast_data = &cast.event.cast;

                match cast.event.cast {
                    shared::event::server::Cast::Teleport(targ) => {
                        let cube = PbrBundle {
                            mesh: meshes.add(Mesh::from(Cuboid { half_size: Vec3::splat(0.5) })),
                            material: materials.add(Color::rgb(0.7, 0.8, 0.9)),
                            transform: Transform::from_translation(targ + Vec3::new(0.0, 0.5, 0.0))
                                .with_scale(Vec3::new(0.5, 20.0, 0.5)),
                            ..Default::default()
                        };

                        commands.spawn((
                            cube,
                            DespawnTime(Timer::new(
                                cast_data.get_skill_info().get_free_point(),
                                TimerMode::Once,
                            )),
                            TpCube(cast.event.cast_id),
                        ));
                    }
                    // Most skills do nothing when initially cast except start an animation
                    _ => {}
                }
                if is_us {
                    // interp locally
                    continue;
                }

                commands.entity(casting_ent).insert((
                    AnimationTimer(Timer::new(
                        cast_data.get_skill_info().get_total_duration(),
                        TimerMode::Once,
                    )),
                    CastPointTimer(Timer::new(
                        cast_data.get_skill_info().get_cast_point(),
                        TimerMode::Once,
                    )),
                    CastNetId(cast.event.cast_id),
                    cast_data.clone(),
                ));
            }
        }
    }
}

fn on_someone_hit(
    mut someone_hit: ERFE<BulletHit>,
    all_plys: Query<(&NetEntId, &Transform, Option<&PlayerName>, Has<Player>), With<AnyUnit>>,
    //mut notifs: EventWriter<Notification>,
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
            // This happens when the bullet hit packet arrives before the "spawn bullet" packet.
            // TODO! maybe add this to a queue of hit events that we poll every frame until we find
            // the matching bullet
            // eg. `Local<Vec<BulletHit>>`
            None => return trace!(?hit.event.bullet, "Unknown bullet"),
        };

        let mut attacker_name = None;
        let mut defender_name = None;

        for (ply_id, _ply_tfm, p_name, is_us) in &all_plys {
            if ply_id == &hit.event.player {
                defender_name = p_name.map(|x| x.0.clone());
                if is_us {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sounds/hit.ogg"),
                        ..default()
                    });
                    debug!("We got hit!");
                } else {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sounds/hitmarker.ogg"),
                        ..default()
                    });
                }
            }

            if ply_id == &bullet_caster_id {
                attacker_name = p_name.map(|x| x.0.clone());
            }
        }

        match (attacker_name, defender_name) {
            (Some(atk), Some(def)) => {
                debug!(?atk, ?def, "Hit!");
            }
            (Some(atk), None) => {
                debug!(?atk, "Player hit NPC");
            }
            (None, Some(def)) => {
                debug!(?def, "NPC hit player");
            }
            (None, None) => {
                debug!("NPC hit NPC");
            }
        }
    }
}
