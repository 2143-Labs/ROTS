use std::time::Duration;

use bevy::prelude::*;
use bevy_time::common_conditions::on_timer;
use shared::{
    animations::DoCast, event::{
        client::{SomeoneCast, SomeoneMoved, SpawnUnit}, server::Cast, spells::AIType, NetEntId
    }, netlib::{
        send_event_to_server, send_event_to_server_batch, EventToClient, EventToServer,
        ServerResources,
    }, unit::{AttackIntention, MovementIntention}, AnyUnit, Controlled
};

use crate::{ConnectedPlayerName, PlayerEndpoint, ServerState};

pub struct NPCPlugin;
impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUnit>()
            .add_event::<AIFinishAttack>()
            .add_systems(
                Update,
                (on_ai_tick, apply_npc_movement_intents, on_unit_spawn, on_ai_finish_attack)
                    //.run_if(on_timer(Duration::from_millis(10)))
                    .run_if(in_state(ServerState::Running)),
            )
            .add_systems(
                Update,
                (send_networked_npc_move)
                    .run_if(in_state(ServerState::Running))
                    .run_if(on_timer(Duration::from_millis(50))),
            );
    }
}

fn on_unit_spawn(
    mut spawns: EventReader<SpawnUnit>,
    mut commands: Commands,
    //players: Query<(Entity, &Transform, &NetEntId, &ConnectedPlayerName)>,
    sr: Res<ServerResources<EventToServer>>,
    clients: Query<&PlayerEndpoint, With<AnyUnit>>,
) {
    for spawn in spawns.read() {
        let mut base = commands.spawn((
            AnyUnit,
            MovementIntention(Vec2::ZERO),
            AttackIntention::None,
            spawn.data.ent_id,
            spawn.data.health,
            spawn.data.transform,
        ));

        match &spawn.data.unit {
            shared::event::UnitType::Player { name } => {
                // This is likely invalid because on_player_connect also inserts the commands to
                // spawn the player.
                error!(?name, "Sent a SpawnUnit event containing a new player");
                return;
            }
            shared::event::UnitType::NPC { npc_type } => {
                base.insert((npc_type.clone(), npc_type.get_ai_component()));
            }
        };

        let event = EventToClient::SpawnUnit(SpawnUnit {
            data: spawn.data.clone(),
        });
        for endpoint in &clients {
            send_event_to_server(&sr.handler, endpoint.0, &event);
        }
    }
}
// Every unit has the same hitbox size for now
const HITBOX_SIZE: f32 = 2.0;
const HITBOX_SIZE_SQ: f32 = HITBOX_SIZE * HITBOX_SIZE;

#[derive(Event, Debug)]
struct AIFinishAttack(DoCast);

fn on_ai_tick(
    mut ai_units: Query<(&NetEntId, &mut Transform, &mut MovementIntention, &mut AttackIntention, &AIType), Without<Controlled>>,
    mut ai_unit_finished_attack: EventWriter<AIFinishAttack>,
    non_ai: Query<(&Transform, &MovementIntention), With<Controlled>>,
    time: Res<Time>,
) {
    let all_player_positions: Vec<(&Transform, &MovementIntention)> = non_ai.iter().collect();
    for (ne_id, mut unit_tfm, mut unit_mi, mut unit_atk, ai_type) in &mut ai_units {
        match ai_type {
            AIType::None => {}
            AIType::WalkToNearestPlayer => {
                let closest = all_player_positions.iter().reduce(|acc, x| {
                    let dist_old = unit_tfm.translation.distance(acc.0.translation);
                    let dist_new = unit_tfm.translation.distance(x.0.translation);
                    if dist_old < dist_new {
                        acc
                    } else {
                        x
                    }
                });

                if let Some(closest) = closest {
                    let target = closest.0.translation.xz();
                    let our_pos = unit_tfm.translation.xz();

                    let dir = target - our_pos;
                    // Movement Intentions
                    if dir.length_squared() < (HITBOX_SIZE_SQ + 0.5) {
                        unit_mi.0 = Vec2::ZERO;
                    } else {
                        let dir = dir.normalize();
                        unit_mi.0 = dir * 0.25;
                        unit_tfm.rotation = Quat::from_rotation_y(
                            -(dir.y.atan2(dir.x)) - std::f32::consts::PI / 2.,
                        );
                    }

                    // Slightly larger attack radius
                    if dir.length_squared() < (HITBOX_SIZE_SQ + 3.0) {
                        match unit_atk.as_mut() {
                            AttackIntention::None => {
                                const AUTO_ATTACK_SPEED: f32 = 0.5;
                                *unit_atk = AttackIntention::AutoAttack(Timer::new(Duration::from_secs_f32(AUTO_ATTACK_SPEED), TimerMode::Repeating));
                            },
                            AttackIntention::AutoAttack(dur) => {
                                dur.tick(time.delta());
                                // The NPC has attacked
                                for _time in 0..dur.times_finished_this_tick() {
                                    ai_unit_finished_attack.send(AIFinishAttack(DoCast(SomeoneCast {
                                        caster_id: *ne_id,
                                        cast_id: NetEntId::random(),
                                        cast: Cast::Melee,
                                    })));
                                }
                            },
                        }
                    } else {
                        *unit_atk = AttackIntention::None;
                    }
                } else if unit_mi.0.length_squared() > 0.0 {
                    unit_mi.0 = Vec2::ZERO;
                    // TODO units will all stop attacking when there is no target
                    *unit_atk = AttackIntention::None;
                }
            }
        }
    }
}

fn on_ai_finish_attack(
    mut ev: EventReader<AIFinishAttack>,
    mut evw: EventWriter<DoCast>,
) {
    for e in ev.read() {
        info!(?e);
        evw.send(e.0.clone());
    }
}


fn apply_npc_movement_intents(
    mut npcs: Query<(&mut Transform, &mut MovementIntention, &AttackIntention), (With<AIType>, Without<Controlled>)>,
    non_ai: Query<(&Transform, &MovementIntention), With<Controlled>>,
    time: Res<Time>,
) {
    // Apply all the movement
    for (mut ply_tfm, ply_intent, attack_intent) in &mut npcs {
        let delta_target =
            Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * 25.0 * time.delta_seconds();
        ply_tfm.translation += delta_target;
        if let AttackIntention::AutoAttack(timer) = attack_intent {
            // TODO path taken by unit is equal to
            //  y = t % Q
            // where Q is the timer repeat duration (0.5)
            ply_tfm.translation.y = timer.elapsed_secs() * 1.5;
        }
    }

    // Now, if anyone is overlapping, we try to gently move them away from eachother
    for _i in 0..3 {
        // Remember if we have done any corrections this frame
        let mut has_corrected = false;

        // First, we need to collection all the positions of all units so we can check.
        let mut all_npc_positions = Vec::with_capacity(npcs.iter().len());
        for (ply_tfm, _, _) in &npcs {
            all_npc_positions.push(ply_tfm.translation);
        }
        let all_player_positions: Vec<_> = non_ai.iter().map(|x| x.0.translation).collect();

        // Call this function to get an iterator
        let all_positions = || all_npc_positions.iter().chain(all_player_positions.iter());

        // Now, check for all other collisions
        // TODO: This is O(n^3). If len npcs > 1000, maybe log a warning that we need to rewrite
        // this?
        for (mut ply_tfm, mut ply_intent, _) in &mut npcs {
            let new_pos = ply_tfm.translation;
            for &other_unit in all_positions() {

                let dist = new_pos.xz().distance_squared(other_unit.xz());
                if dist <= 0.005 {
                    // This is too close, just let it stay here (minecraft mob stacking style)
                    continue;
                } else if dist <= HITBOX_SIZE_SQ {
                    // Now, calculate the offset we need to apply in 2d space
                    let diff = new_pos - other_unit;
                    let diff_xz = diff.xz();
                    let correction_2d = (diff_xz.normalize() * HITBOX_SIZE) - diff_xz;

                    // Apply that 2d correction to 3d space
                    // Ideal movement would be 0.5 because each unit will be moved away equally from eachother. but we do more than one correction to make it look nice
                    let correction = correction_2d.xyy() * Vec3::new(1.0, 0.0, 1.0) * 0.25;
                    ply_tfm.translation += correction;

                    // We are collding: stop trying to appear as if we are moving
                    *ply_intent = MovementIntention(Vec2::ZERO);

                    has_corrected = true;
                }
            }
        }

        if !has_corrected {
            return;
        }
    }
    debug!("Too many collisions this frame");
}

fn send_networked_npc_move(
    npcs: Query<
        (&Transform, &MovementIntention, &AttackIntention, &NetEntId),
        (
            With<AIType>,
            Or<(Changed<Transform>, Changed<MovementIntention>, Changed<AttackIntention>)>,
        ),
    >,
    clients: Query<&PlayerEndpoint, With<ConnectedPlayerName>>,
    sr: Res<ServerResources<EventToServer>>,
) {
    let mut all_events = vec![];
    for (&movement, mi, attack_intent, &id) in &npcs {
        all_events.extend([
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::SetTransform(movement),
            }),
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::Move2d(mi.0),
            }),
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::AttackIntent(attack_intent.clone()),
            }),
        ]);
    }

    if !all_events.is_empty() {
        for event_list in all_events.chunks(250) {
            for endpoint in &clients {
                send_event_to_server_batch(&sr.handler, endpoint.0, event_list);
            }
        }
    }
}
