use bevy::prelude::*;
use shared::event::{NetEntId, client::SpawnNPC};

use crate::ServerState;

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
    //clients: Query<&PlayerEndpoint, With<AnyPlayer>>,
) {
    for spawn in spawns.read() {
        let eid = NetEntId(rand::random());
        commands.spawn((
            eid,
            Transform::from_translation(spawn.location),
            spawn.npc.clone(),
        ));
    }
}
