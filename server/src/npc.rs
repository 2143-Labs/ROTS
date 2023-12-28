use bevy::prelude::*;
use shared::{
    event::client::SpawnUnit,
    netlib::{send_event_to_server, EventToClient, EventToServer, ServerResources},
    AnyPlayer,
};

use crate::{PlayerEndpoint, ServerState};

pub struct NPCPlugin;
impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUnit>().add_systems(
            Update,
            (on_unit_spawn).run_if(in_state(ServerState::Running)),
        );
    }
}

fn on_unit_spawn(
    mut spawns: EventReader<SpawnUnit>,
    mut commands: Commands,
    //players: Query<(Entity, &Transform, &NetEntId, &ConnectedPlayerName)>,
    sr: Res<ServerResources<EventToServer>>,
    clients: Query<&PlayerEndpoint, With<AnyPlayer>>,
) {
    for spawn in spawns.read() {
        match &spawn.data.unit {
            shared::event::UnitType::Player { name } => {
                //commands.spawn((
                    //ConnectedPlayerName { name },
                    //new_player_data.ent_id,
                    //new_player_data.health,
                    //new_player_data.transform,
                    //PlayerEndpoint(player.endpoint),
                    //// Transform component used for generic systems
                    //shared::AnyPlayer,
                //));
                // This is likely invalid because on_player_connect also inserts the commands to
                // spawn the player.
                error!(?name, "Sent a SpawnUnit event containing a new player");
                return;
            },
            shared::event::UnitType::NPC { npc_type } => {
                commands.spawn((
                    spawn.data.transform,
                    spawn.data.ent_id,
                    spawn.data.health,
                    npc_type.clone(),
                ));
            },
        };

        let event = EventToClient::SpawnUnit(SpawnUnit {
            data: spawn.data.clone(),
        });
        for endpoint in &clients {
            send_event_to_server(&sr.handler, endpoint.0, &event);
        }
    }
}
