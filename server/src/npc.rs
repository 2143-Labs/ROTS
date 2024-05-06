use std::time::Duration;

use bevy::prelude::*;
use bevy_time::common_conditions::on_timer;
use shared::{
    event::{
        client::{SomeoneMoved, SpawnUnit},
        spells::AIType,
        NetEntId,
    },
    netlib::{
        send_event_to_server, send_event_to_server_batch, EventToClient, EventToServer,
        ServerResources,
    },
    unit::MovementIntention,
    AnyUnit, Controlled,
};

use crate::{ConnectedPlayerName, PlayerEndpoint, ServerState};

pub struct NPCPlugin;
impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUnit>()
            .add_systems(
                Update,
                (on_ai_tick, apply_npc_movement_intents, on_unit_spawn)
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

fn on_ai_tick(
    mut ai_units: Query<(&mut Transform, &mut MovementIntention, &AIType), Without<Controlled>>,
    non_ai: Query<(&Transform, &MovementIntention), With<Controlled>>,
) {
    let all_player_positions: Vec<(&Transform, &MovementIntention)> = non_ai.iter().collect();
    for (mut unit_tfm, mut unit_mi, ai_type) in &mut ai_units {
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
                    if dir.length_squared() > 1.0 {
                        let dir = dir.normalize();
                        unit_mi.0 = dir * 0.25;
                        unit_tfm.rotation = Quat::from_rotation_y(
                            -(dir.y.atan2(dir.x)) - std::f32::consts::PI / 2.,
                        );
                    } else {
                        unit_mi.0 = Vec2::ZERO;
                    }
                } else if unit_mi.0.length_squared() > 0.0 {
                    unit_mi.0 = Vec2::ZERO;
                }
            }
        }
    }
}

fn apply_npc_movement_intents(
    mut npcs: Query<(&mut Transform, &MovementIntention), (With<AIType>, Without<Controlled>)>,
    non_ai: Query<(&Transform, &MovementIntention), With<Controlled>>,
    time: Res<Time>,
) {
    // Apply all the movement
    for (mut ply_tfm, ply_intent) in &mut npcs {
        let delta_target = Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * 25.0 * time.delta_seconds();
        ply_tfm.translation += delta_target;
    }

    for i in 0..1 {
        let mut has_corrected = false;

        let mut all_npc_positions = Vec::with_capacity(npcs.iter().len());
        for (ply_tfm, _) in &npcs {
            all_npc_positions.push(ply_tfm.translation);
        }

        //let ent = npcs.single().0;
        let all_player_positions: Vec<_> = non_ai.iter().map(|x| x.0.translation).collect();
        let all_positions = || all_npc_positions.iter().chain(all_player_positions.iter());

        // Look for any corrections we might need to do
        for (mut ply_tfm, ply_intent) in &mut npcs {
            let pos_delta = Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * 25.0 * time.delta_seconds();
            let mut correction = Vec3::ZERO;
            let new_pos = ply_tfm.translation;
            let old_pos = new_pos - pos_delta;
            for &other_unit in all_positions() {
                const HITBOX_SIZE: f32 = 5.0;
                if new_pos == other_unit {
                    // This is us
                    continue;
                } else if new_pos.distance_squared(other_unit) <= HITBOX_SIZE {
                    let diff = other_unit - new_pos;
                    let diff_xz = diff.xz();
                    let correction_2d = (diff_xz.normalize() * HITBOX_SIZE) - diff_xz;
                    error!("unit correcton {diff_xz}, {correction_2d}");
                    correction += correction_2d.xyy() * Vec3::new(1.0, 0.0, 1.0);
                    has_corrected = true;
                }
            }

            ply_tfm.translation = ply_tfm.translation + correction;
        }

        if !has_corrected {
            if i >= 1 {
                warn!(i, "Movement collision loops this frame");
            }
            break;
        }
    }
}

fn send_networked_npc_move(
    npcs: Query<
        (&Transform, &MovementIntention, &NetEntId),
        (
            With<AIType>,
            Or<(Changed<Transform>, Changed<MovementIntention>)>,
        ),
    >,
    clients: Query<&PlayerEndpoint, With<ConnectedPlayerName>>,
    sr: Res<ServerResources<EventToServer>>,
) {
    let mut all_events = vec![];
    for (&movement, mi, &id) in &npcs {
        all_events.extend([
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::SetTransform(movement),
            }),
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::Move2d(mi.0),
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
