use std::{
    mem::{discriminant, Discriminant},
    time::Duration,
};

use bevy::{prelude::*, utils::HashSet};
use shared::{
    animations::{AnimationTimer, CastNetId, CastPointTimer, DoCast, DoDamage},
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin},
    event::{
        client::{
            BulletHit, SomeoneCast, SomeoneUpdateComponent, SpawnInteractable, UnitDie,
            YourCastResult,
        },
        server::Cast,
        spells::ShootingData,
        NetEntId, ERFE,
    },
    interactable::Interactable,
    netlib::{send_event_to_server, EventToClient, EventToServer, ServerResources},
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
            .add_event::<DoSpawnInteractable>()
            .add_event::<DoCast>()
            .add_event::<DoDamage>()
            .insert_resource(HitList::default())
            .add_systems(
                Update,
                (
                    on_player_try_cast,
                    hit,
                    check_collision,
                    on_die,
                    shared::animations::systems::tick_casts,
                    do_cast,
                    spawn_interactable,
                    unit_damaged,
                    tick_spell_proj,
                )
                    .run_if(in_state(ServerState::Running)),
            );
    }
}

#[derive(Component, Debug)]
pub(crate) struct PlayerCooldown(pub Discriminant<Cast>, pub NetEntId);

#[derive(Component, Debug)]
pub(crate) struct SpellProj(pub Timer, pub Cast);

#[derive(Component, Debug)]
pub(crate) struct SpellTarget(pub NetEntId);

fn tick_spell_proj(
    mut projectiles: Query<(Entity, &mut SpellProj, &SpellTarget)>,
    mut damage_events: EventWriter<DoDamage>,
    time: Res<Time<Virtual>>,
    mut commands: Commands,
) {
    for (ent, mut sp, target_id) in &mut projectiles {
        sp.0.tick(time.delta());
        if sp.0.finished() {
            damage_events.send(DoDamage(target_id.0, sp.1.get_damage()));
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn do_cast(
    mut do_cast: EventReader<DoCast>,
    mut commands: Commands,
    all_unit_locations: Query<(&NetEntId, &Transform)>,
    _time: Res<Time<Virtual>>,
    mut damage_events: EventWriter<DoDamage>,
) {
    for DoCast(cast) in do_cast.read() {
        trace!(?cast, "Cast has completed");

        commands.spawn((
            PlayerCooldown(discriminant(&cast.cast), cast.caster_id),
            DespawnTime(Timer::new(
                cast.cast.get_skill_info().cooldown,
                TimerMode::Once,
            )),
        ));

        match cast.cast {
            Cast::Teleport(_) => {} // TODO
            Cast::Shoot(ref shot_data) => {
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
            Cast::Aoe(loc) => {
                for (other_unit_ent_id, other_unit_tfm) in &all_unit_locations {
                    if other_unit_tfm.translation.distance(loc) < 25.0
                        && &cast.caster_id != other_unit_ent_id
                    {
                        //TODO also check angle of attach
                        damage_events.send(DoDamage(*other_unit_ent_id, cast.cast.get_damage()));
                    }
                }
            }
            Cast::Melee => {
                for (unit_ent_id, unit_tfm) in &all_unit_locations {
                    // find everything in an aoe around the caster
                    if unit_ent_id == &cast.caster_id {
                        for (other_unit_ent_id, other_unit_tfm) in &all_unit_locations {
                            if other_unit_tfm.translation.distance(unit_tfm.translation) < 5.0
                                && unit_ent_id != other_unit_ent_id
                            {
                                //TODO also check angle of attach
                                damage_events
                                    .send(DoDamage(*other_unit_ent_id, cast.cast.get_damage()));
                            }
                        }
                    }
                }
            }
            Cast::ShootTargeted(_, target_net_ent_id) => {
                commands.spawn((
                    //bullets have a net ent id + a caster.
                    cast.cast_id,
                    CasterNetId(cast.caster_id),
                    // TODO hardcoded proj duration
                    SpellProj(
                        Timer::new(Duration::from_secs(1), TimerMode::Once),
                        cast.cast.clone(),
                    ),
                    SpellTarget(target_net_ent_id),
                ));
            }
            Cast::Buff => {
                for (unit_ent_id, unit_tfm) in &all_unit_locations {
                    // find everything in an aoe around the caster
                    if unit_ent_id == &cast.caster_id {
                        for (other_unit_ent_id, other_unit_tfm) in &all_unit_locations {
                            if other_unit_tfm.translation.distance(unit_tfm.translation) < 25.0 {
                                //TODO buff the units here
                                warn!(?other_unit_ent_id, "Buff was cast on");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn on_player_try_cast(
    mut casts: ERFE<shared::event::server::Cast>,
    endpoint_mapping: Res<EndpointToNetId>,
    clients: Query<(&PlayerEndpoint, &NetEntId)>,
    casting_units: Query<(Entity, &NetEntId, Option<(&AnimationTimer, &Cast)>), With<AnyUnit>>,
    cooldowns: Query<(&PlayerCooldown, &DespawnTime)>,
    sr: Res<ServerResources<EventToServer>>,
    mut commands: Commands,
) {
    'next_cast: for cast in casts.read() {
        if let Some(caster_net_id) = endpoint_mapping.map.get(&cast.endpoint) {
            // if it's on cd, deny it and don't tell anyone else.
            for (cd, time_left) in &cooldowns {
                if cd.1 == *caster_net_id && discriminant(&cast.event) == cd.0 {
                    debug!(?cd, "denied cast for cooldown");
                    let event =
                        EventToClient::YourCastResult(YourCastResult::No(time_left.0.remaining()));
                    send_event_to_server(&sr.handler, cast.endpoint, &event);
                    continue 'next_cast;
                }
            }

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

            // tell the client they are ok to continue their animation
            let event = EventToClient::YourCastResult(YourCastResult::Ok(new_cast_id));
            send_event_to_server(&sr.handler, cast.endpoint, &event);

            for (casting_ent, net_ent_id, _current_cast) in &casting_units {
                if net_ent_id == caster_net_id {
                    trace!(?net_ent_id, ?cast.event, "Adding the cast to the entity");
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

fn unit_damaged(
    mut damage_events: EventReader<DoDamage>,
    mut death: EventWriter<UnitDie>,
    clients: Query<&PlayerEndpoint>,
    mut unit: Query<(&NetEntId, &mut Health), With<AnyUnit>>,
    sr: Res<ServerResources<EventToServer>>,
) {
    for DoDamage(net_ent_id, damage) in damage_events.read() {
        for (unit_net_id, mut ply_hp) in &mut unit {
            if unit_net_id == net_ent_id {
                debug!(?net_ent_id, ?damage, "Unit took damage");
                // Remove the damage from their hp & update all connected clients
                ply_hp.0 = ply_hp.0.saturating_sub(*damage as _);

                let hp_event = EventToClient::SomeoneUpdateComponent(SomeoneUpdateComponent {
                    id: *net_ent_id,
                    update: shared::event::spells::UpdateSharedComponent::Health(*ply_hp),
                });

                for c_net_client in &clients {
                    send_event_to_server(&sr.handler, c_net_client.0, &hp_event)
                }
                if ply_hp.0 <= 0 {
                    death.send(UnitDie { id: *net_ent_id });
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct HitList(HashSet<BulletHit>);

fn hit(
    mut ev_r: EventReader<BulletHit>,
    mut damage_events: EventWriter<DoDamage>,
    clients: Query<&PlayerEndpoint>,
    mut unit: Query<&NetEntId, With<AnyUnit>>,
    sr: Res<ServerResources<EventToServer>>,
    mut hit_list: ResMut<HitList>,
) {
    for e in ev_r.read() {
        if hit_list.0.contains(e) {
            continue;
        }

        hit_list.0.insert(e.clone());

        let bullet_damage = { Cast::Shoot(ShootingData::default()).get_damage() };

        for ent_id in &mut unit {
            if ent_id == &e.player {
                damage_events.send(DoDamage(*ent_id, bullet_damage));
            }
        }

        for c_net_client in &clients {
            send_event_to_server(
                &sr.handler,
                c_net_client.0,
                &EventToClient::BulletHit(e.clone()),
            );
        }
    }
}

#[derive(Event)]
struct DoSpawnInteractable(Vec3);
fn spawn_interactable(
    mut do_spawns: EventReader<DoSpawnInteractable>,
    sr: Res<ServerResources<EventToServer>>,
    clients: Query<&PlayerEndpoint>,
    mut commands: Commands,
) {
    for spawn in do_spawns.read() {
        let new_id = NetEntId(rand::random());
        let event = EventToClient::SpawnInteractable(SpawnInteractable {
            id: new_id,
            location: spawn.0,
        });

        //warn!("Spawn interactable ev!");
        commands.spawn((
            Transform::from_translation(spawn.0),
            new_id,
            Interactable,
            // TODO look for interactions
        ));

        for c_net_client in &clients {
            send_event_to_server(&sr.handler, c_net_client.0, &event)
        }
    }
}

fn on_die(
    mut death: EventReader<UnitDie>,
    sr: Res<ServerResources<EventToServer>>,
    ents: Query<(Entity, &NetEntId, &Transform), With<AnyUnit>>,
    clients: Query<&PlayerEndpoint>,
    mut commands: Commands,
    mut do_spawns: EventWriter<DoSpawnInteractable>,
) {
    for death in death.read() {
        let event = EventToClient::UnitDie(death.clone());
        for (unit_ent, unit_ent_id, unit_tfm) in &ents {
            if unit_ent_id == &death.id {
                do_spawns.send(DoSpawnInteractable(unit_tfm.translation));
                //warn!("Spawn interactable!");
                commands.entity(unit_ent).despawn_recursive();
            }
        }

        for c_net_client in &clients {
            send_event_to_server(&sr.handler, c_net_client.0, &event)
        }
    }
}
