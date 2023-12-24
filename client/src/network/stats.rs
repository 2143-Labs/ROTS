use bevy::prelude::*;
use shared::{
    event::{client::SomeoneUpdateComponent, NetEntId, ERFE},
    stats::Health,
    AnyPlayer,
};

use crate::{
    cameras::notifications::Notification,
    player::{Player, PlayerName},
    states::GameState,
};

pub struct StatsNetworkPlugin;

impl Plugin for StatsNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_someone_update_stats, on_die).run_if(in_state(GameState::ClientConnected)),
        );
    }
}

fn on_someone_update_stats(
    mut stat_update: ERFE<SomeoneUpdateComponent>,
    mut players: Query<(&NetEntId, &mut Health, &PlayerName), With<AnyPlayer>>,
) {
    for update in stat_update.read() {
        for (ply_ent, mut ply_hp, name) in &mut players {
            warn!(?name, "Someone changed hp??");
            if ply_ent == &update.event.id {
                warn!(?update.event);
                match update.event.update {
                    shared::event::spells::UpdateSharedComponent::Health(hp) => {
                        *ply_hp = hp;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub enum HPIndicator {
    HP,
    Deaths,
}

fn on_die(
    mut notifs: EventWriter<Notification>,
    mut me: Query<
        (&mut Transform, &mut Health, Has<Player>, &PlayerName),
        (With<AnyPlayer>, Changed<Health>),
    >,
    mut hp_text: Query<(&mut Text, &HPIndicator)>,
    mut total_deaths: Local<u32>,
) {
    for (mut tfm, mut hp, is_us, PlayerName(name)) in &mut me {
        warn!("Someone changed hp");
        if hp.0 == 0 {
            // Log a death and reset
            *hp = Health::default();
            tfm.translation = Vec3::new(0.0, 1.0, 0.0);

            info!(?name);
            if is_us {
                *total_deaths += 1;
                notifs.send(Notification(format!(
                    "We died! Total Deaths: {}",
                    *total_deaths
                )));
            } else {
                notifs.send(Notification(format!("{name} died!")));
            }
        }

        if is_us {
            for (mut text, ind_type) in &mut hp_text {
                let text_section = &mut text.sections[0].value;
                match ind_type {
                    HPIndicator::HP => {
                        *text_section = format!("{} HP", hp.0);
                    }
                    HPIndicator::Deaths => match *total_deaths {
                        0 => {}
                        deaths => {
                            *text_section = format!("Deaths: {deaths}");
                        }
                    },
                }
            }
            //match *total_deaths {
            //0 => {
            //*text_section = format!("HP: {}", hp.0);
            //},
            //deaths => {
            //*text_section = format!("(D {deaths}) HP: {}", hp.0);
            //},
            //}
        }
    }
}
