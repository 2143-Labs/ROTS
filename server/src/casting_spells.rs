use std::time::Duration;

use bevy::{prelude::*, utils::HashSet};
use shared::{
    animations::AnimationTimer,
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin},
    event::{
        client::{BulletHit, SomeoneCast, SomeoneUpdateComponent, UnitDie},
        server::Cast,
        spells::{ShootingData, NPC},
        NetEntId, ERFE,
    },
    netlib::{
        send_event_to_server, send_event_to_server_batch, EventToClient, EventToServer,
        ServerResources,
    },
    stats::Health,
    AnyUnit,
};

use crate::{EndpointToNetId, PlayerEndpoint, ServerState};

pub struct CastingPlugin;

impl Plugin for CastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedCastingPlugin)
            .add_event::<BulletHit>()
            .add_event::<UnitDie>()
            .add_event::<DoCast>()
            .insert_resource(HitList::default())
            .add_systems(
                Update,
                (
                    on_player_try_cast,
                    hit,
                    check_collision,
                    on_die,
                    tick_casts,
                    do_cast,
                )
                    .run_if(in_state(ServerState::Running)),
            );
    }
}

//TODOs:
// - [ ] player can cancel casting their stuff
// - [ ] server can stop a player from casting
//
///go until the cast point and then do the actual effect
fn tick_casts(
    mut casting_units: Query<(
        Entity,
        &NetEntId,
        &mut CastPointTimer,
        &mut AnimationTimer,
        &Cast,
        &CastNetId,
    )>,
    mut commands: Commands,
    mut do_cast: EventWriter<DoCast>,
    time: Res<Time<Virtual>>,
) {
    for (_unit_ent, _net_ent_id, mut cast_timer, mut anim_timer, cast, cast_net_id) in
        &mut casting_units
    {
        cast_timer.0.tick(time.delta());
        anim_timer.0.tick(time.delta());

        if cast_timer.0.finished() {
            do_cast.send(DoCast(SomeoneCast {
                caster_id: *_net_ent_id,
                cast_id: cast_net_id.0,
                cast: cast.clone(),
            }));
        }

        if anim_timer.0.finished() {
            commands
                .entity(_unit_ent)
                .remove::<AnimationTimer>()
                .remove::<CastPointTimer>()
                .remove::<Cast>()
                .remove::<CastNetId>();
        }
    }
}

#[derive(Event)]
struct DoCast(SomeoneCast);

fn do_cast(mut do_cast: EventReader<DoCast>, mut commands: Commands) {
    for DoCast(cast) in do_cast.read() {
        match cast.cast {
            shared::event::server::Cast::Teleport(_) => {} // TODO
            shared::event::server::Cast::Shoot(ref shot_data) => {
                commands.spawn((
                    Transform::from_translation(shot_data.shot_from),
                    shot_data.clone(),
                    //bullets have a net ent id + a caster.
                    cast.cast_id,
                    CasterNetId(cast.caster_id),
                    DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                    // TODO Add a netentid for referencing this item later
                ));
            }
            _ => {}
        }
    }
}

/// TODO
fn check_cancelled_casts() {}

#[derive(Component)]
struct CastPointTimer(Timer);

#[derive(Component)]
struct CastNetId(NetEntId);

fn on_player_try_cast(
    mut casts: ERFE<shared::event::server::Cast>,
    endpoint_mapping: Res<EndpointToNetId>,
    clients: Query<(&PlayerEndpoint, &NetEntId)>,
    casting_units: Query<(Entity, &NetEntId, Option<(&AnimationTimer, &Cast)>), With<AnyUnit>>,
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

            for (casting_ent, net_ent_id, _current_cast) in &casting_units {
                if net_ent_id == caster_net_id {
                    commands.entity(casting_ent).insert((
                        AnimationTimer(Timer::new(
                            cast.event.get_skill_info().get_total_duration(),
                            TimerMode::Once,
                        )),
                        CastPointTimer(Timer::new(
                            cast.event.get_skill_info().get_cast_point(),
                            TimerMode::Once,
                        )),
                        CastNetId(new_cast_id),
                        cast.event.clone(),
                    ));
                }
            }
        }
    }
}

fn check_collision(
    bullets: Query<(&NetEntId, &CasterNetId, &Transform), (With<ShootingData>, Without<AnyUnit>)>,
    players: Query<(&NetEntId, &Transform), With<AnyUnit>>,
    mut ev_w: EventWriter<BulletHit>,
) {
    for (b_id, CasterNetId(caster), bullet) in &bullets {
        for (p_id, player) in &players {
            if caster == p_id {
                //you cannot hit yourself
                continue;
            }

            if bullet.translation.distance_squared(player.translation) < 5.0 {
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
    mut death: EventWriter<UnitDie>,
    clients: Query<&PlayerEndpoint>,
    // todo make this into an event
    mut unit: Query<(&NetEntId, &mut Health, Option<&NPC>), With<AnyUnit>>,
    sr: Res<ServerResources<EventToServer>>,
    mut hit_list: ResMut<HitList>,
) {
    for e in ev_r.read() {
        if hit_list.0.contains(e) {
            continue;
        }

        hit_list.0.insert(e.clone());
        let mut hp_event = None;
        for (ent_id, mut ply_hp, npc_data) in &mut unit {
            if ent_id == &e.player {
                ply_hp.0 = ply_hp.0.saturating_sub(1);
                hp_event = Some(EventToClient::SomeoneUpdateComponent(
                    SomeoneUpdateComponent {
                        id: *ent_id,
                        update: shared::event::spells::UpdateSharedComponent::Health(*ply_hp),
                    },
                ));

                if ply_hp.0 <= 0 {
                    death.send(UnitDie { id: *ent_id });
                    // They get respawned on the client
                    if npc_data.is_none() {
                        *ply_hp = Health::default();
                    }
                }
            }
        }

        for c_net_client in &clients {
            if let Some(e) = &hp_event {
                send_event_to_server(&sr.handler, c_net_client.0, e);
            }
            send_event_to_server(
                &sr.handler,
                c_net_client.0,
                &EventToClient::BulletHit(e.clone()),
            );
        }
    }
}

fn on_die(
    mut death: EventReader<UnitDie>,
    sr: Res<ServerResources<EventToServer>>,
    ents: Query<(Entity, &NetEntId), With<AnyUnit>>,
    clients: Query<&PlayerEndpoint>,
    mut commands: Commands,
) {
    let deaths: Vec<EventToClient> = death
        .read()
        .inspect(|x| {
            // If we have a unit with this id, despawn it.
            for (unit_ent, ent_id) in &ents {
                if ent_id == &x.id {
                    commands.entity(unit_ent).despawn_recursive();
                }
            }
        })
        .map(|x| {
            // TODO add a method on EventToX that turns a struct into the wrapped variant
            EventToClient::UnitDie(x.clone())
        })
        .collect();

    if deaths.len() > 0 {
        for c_net_client in &clients {
            send_event_to_server_batch(&sr.handler, c_net_client.0, &deaths)
        }
    }
}
