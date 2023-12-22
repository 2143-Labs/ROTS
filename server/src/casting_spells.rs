use std::time::Duration;

use bevy::{prelude::*, utils::HashSet};
use shared::{
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin},
    event::{
        client::{BulletHit, SomeoneCast},
        spells::ShootingData,
        NetEntId, ERFE,
    },
    netlib::{send_event_to_server, EventToClient, EventToServer, ServerResources},
    AnyPlayer,
};

use crate::{EndpointToNetId, PlayerEndpoint, ServerState};

pub struct CastingPlugin;

impl Plugin for CastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedCastingPlugin)
            .add_event::<BulletHit>()
            .insert_resource(HitList::default())
            .add_systems(
                Update,
                (on_player_try_cast, hit, check_collision).run_if(in_state(ServerState::Running)),
            );
    }
}

fn on_player_try_cast(
    mut casts: ERFE<shared::event::server::Cast>,
    endpoint_mapping: Res<EndpointToNetId>,
    clients: Query<(&PlayerEndpoint, &NetEntId)>,
    sr: Res<ServerResources<EventToServer>>,
    mut commands: Commands,
) {
    for cast in casts.read() {
        if let Some(caster_net_id) = endpoint_mapping.map.get(&cast.endpoint) {
            // if we can cast, then send to all endpoints including us.
            let new_cast_id = NetEntId::random();
            let event = EventToClient::SomeoneCast(SomeoneCast {
                caster_id: *caster_net_id,
                cast_id: new_cast_id,
                cast: cast.event.clone(),
            });
            for (c_net_client, _c_net_ent) in &clients {
                send_event_to_server(&sr.handler, c_net_client.0, &event);
            }

            match cast.event {
                shared::event::server::Cast::Teleport(_) => {} // TODO
                shared::event::server::Cast::Shoot(ref shot_data) => {
                    commands.spawn((
                        Transform::from_translation(shot_data.shot_from),
                        shot_data.clone(),
                        //bullets have a net ent id + a caster.
                        new_cast_id,
                        CasterNetId(*caster_net_id),
                        DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                        // TODO Add a netentid for referencing this item later
                    ));
                }
            }
        }
    }
}

fn check_collision(
    bullets: Query<(&NetEntId, &CasterNetId, &Transform), (With<ShootingData>, Without<AnyPlayer>)>,
    players: Query<(&NetEntId, &Transform), With<AnyPlayer>>,
    mut ev_w: EventWriter<BulletHit>,
) {
    for (b_id, CasterNetId(caster), bullet) in &bullets {
        for (p_id, player) in &players {
            if caster == p_id {
                //you cannot hit yourself
                continue;
            }

            if bullet.translation.distance_squared(player.translation) < 1.0 {
                ev_w.send(BulletHit {
                    bullet: *b_id,
                    player: *p_id,
                });
            }
        }
    }
}

#[derive(Resource, Default)]
struct HitList(HashSet<BulletHit>);

fn hit(
    mut ev_r: EventReader<BulletHit>,
    clients: Query<&PlayerEndpoint>,
    sr: Res<ServerResources<EventToServer>>,
    mut hit_list: ResMut<HitList>,
) {
    for e in ev_r.read() {
        if hit_list.0.contains(e) {
            continue;
        }
        hit_list.0.insert(e.clone());
        for c_net_client in &clients {
            send_event_to_server(
                &sr.handler,
                c_net_client.0,
                &EventToClient::BulletHit(e.clone()),
            );
        }
    }
}
