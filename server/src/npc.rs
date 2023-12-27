use bevy::prelude::*;
use shared::{event::{NetEntId, spells::SpawnNPC, client::NewNPC}, AnyPlayer, netlib::{EventToClient, send_event_to_server, ServerResources, EventToServer}};

use crate::{ServerState, PlayerEndpoint};

pub struct NPCPlugin;
impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnNPC>()
            .add_systems(Update, (on_npc_spawn).run_if(in_state(ServerState::Running)));
    }
}

fn on_npc_spawn(
    mut spawns: EventReader<SpawnNPC>,
    mut commands: Commands,
    //players: Query<(Entity, &Transform, &NetEntId, &ConnectedPlayerName)>,
    sr: Res<ServerResources<EventToServer>>,
    clients: Query<&PlayerEndpoint, With<AnyPlayer>>,
) {
    for spawn in spawns.read() {
        let eid = NetEntId(rand::random());
        commands.spawn((
            eid,
            Transform::from_translation(spawn.location),
            spawn.npc.clone(),
        ));

        let event = EventToClient::NewNPC(NewNPC {
            id: eid,
            spawn_commands: spawn.clone(),
        });
        for endpoint in &clients {
            send_event_to_server(&sr.handler, endpoint.0, &event);
        }
    }
}
