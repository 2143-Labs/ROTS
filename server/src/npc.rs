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
                (on_unit_spawn, on_ai_tick).run_if(in_state(ServerState::Running)),
            )
            .add_systems(
                Update,
                on_npc_move.run_if(on_timer(Duration::from_millis(50))),
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
    let positions: Vec<(&Transform, &MovementIntention)> = non_ai.iter().collect();
    for (mut unit_tfm, mut unit_mi, ai_type) in &mut ai_units {
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
                    unit_tfm.translation.x = closest.0.translation.x;
                    unit_mi.0.x = closest.1 .0.x;
                } else if unit_mi.0.length_squared() > 0.0 {
                    unit_mi.0 = Vec2::ZERO;
                }
            }
        }
    }
}

fn on_npc_move(
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
        let eventa = EventToClient::SomeoneMoved(SomeoneMoved {
            id,
            movement: shared::event::server::ChangeMovement::SetTransform(movement),
        });
        let eventb = EventToClient::SomeoneMoved(SomeoneMoved {
            id,
            movement: shared::event::server::ChangeMovement::Move2d(mi.0),
        });
        let events = &[eventa, eventb];
        for endpoint in &clients {
            send_event_to_server_batch(&sr.handler, endpoint.0, events);
        }
    }
}
