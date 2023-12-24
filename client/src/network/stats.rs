use bevy::prelude::*;
use shared::{
    event::{
        client::SomeoneUpdateComponent,
        NetEntId, ERFE,
    },
    AnyPlayer, stats::Health,
};

use crate::{
    cameras::notifications::Notification,
    player::{Player, PlayerName},
    states::GameState,
};

pub struct StatsNetworkPlugin;

impl Plugin for StatsNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (on_someone_update_stats)
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn on_someone_update_stats(
    mut stat_update: ERFE<SomeoneUpdateComponent>,
    mut players: Query<(&NetEntId, &mut Health, &PlayerName), With<AnyPlayer>>,
){
    for update in stat_update.read() {
        for (ply_ent, mut ply_hp, name) in &mut players {
            warn!(?name, "Someone changed hp??");
            if ply_ent == &update.event.id {
                warn!(?update.event);
                match update.event.update {
                    shared::event::spells::UpdateSharedComponent::Health(hp) => {
                        *ply_hp = hp;
                    },
                }
            }
        }
    }
}
