use std::time::Duration;

use bevy::prelude::*;
use shared::{event::{ERFE, NetEntId, client::SomeoneCast}, netlib::{ServerResources, EventToServer, send_event_to_server, EventToClient}, Config, casting::{DespawnTime, SharedCastingPlugin}};

use crate::{EndpointToNetId, PlayerEndpoint, ServerState, ConnectedPlayerName};

pub struct CastingPlugin;

impl Plugin for CastingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SharedCastingPlugin)
            .add_systems(Update, (
                on_player_try_cast,
            ).run_if(in_state(ServerState::Running)))
                ;
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
            let event = EventToClient::SomeoneCast(SomeoneCast {
                id: *caster_net_id,
                cast: cast.event.clone(),
            });
            for (c_net_client, _c_net_ent) in &clients {
                send_event_to_server(&sr.handler, c_net_client.0, &event);
            }


            match cast.event {
                shared::event::server::Cast::Teleport(_) => {}, // TODO
                shared::event::server::Cast::Shoot(ref shot_data) => {
                    commands.spawn((
                        Transform::from_translation(shot_data.shot_from),
                        shot_data.clone(),
                        NetEntId::random(),
                        DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                        // TODO Add a netentid for referencing this item later
                    ));
                }
            }
        }
    }
}
