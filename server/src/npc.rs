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
    mut ai_units: Query<(&Transform, &mut MovementIntention, &AIType), Without<Controlled>>,
    non_ai: Query<(&Transform, &MovementIntention), With<Controlled>>,
) {
    let positions: Vec<(&Transform, &MovementIntention)> = non_ai.iter().collect();
    for (unit_tfm, mut unit_mi, ai_type) in &mut ai_units {
        match ai_type {
            AIType::None => {}
            AIType::WalkToNearestPlayer => {
                let closest = positions.iter().reduce(|acc, x| {
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
                        unit_mi.0 = dir.normalize() * 0.25;
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
    mut npcs: Query<(&mut Transform, &MovementIntention), With<AIType>>,
    time: Res<Time>,
) {
    for (mut ply_tfm, ply_intent) in &mut npcs {
        ply_tfm.translation +=
            Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * 25.0 * time.delta_seconds();
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
    for (&movement, mi, &id) in &npcs {
        let events = &[
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::SetTransform(movement),
            }),
            EventToClient::SomeoneMoved(SomeoneMoved {
                id,
                movement: shared::event::server::ChangeMovement::Move2d(mi.0),
            }),
        ];
        for endpoint in &clients {
            send_event_to_server_batch(&sr.handler, endpoint.0, events);
        }
    }
}
