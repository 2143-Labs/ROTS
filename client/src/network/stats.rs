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
            (on_someone_update_stats, on_hp_change, update_hp_bar)
                .run_if(in_state(GameState::ClientConnected)),
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

fn on_hp_change(
    mut notifs: EventWriter<Notification>,
    mut players: Query<
        (&mut Transform, &mut Health, Has<Player>, &PlayerName),
        (With<AnyPlayer>, Changed<Health>),
    >,
    mut hp_text: Query<(&mut Text, &HPIndicator)>,
    mut total_deaths: Local<u32>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (mut tfm, mut hp, is_us, PlayerName(name)) in &mut players {
        // die
        if hp.0 == 0 {
            *hp = Health::default();

            // play death sound
            commands.spawn((
                TransformBundle::from_transform(Transform::from_translation(tfm.translation)),
                AudioBundle {
                    source: asset_server.load("sounds/death.ogg"),
                    settings: PlaybackSettings::DESPAWN.with_spatial(true),
                    ..default()
                },
            ));

            tfm.translation = Vec3::new(0.0, 1.0, 0.0);

            if is_us {
                *total_deaths += 1;
            } else {
                notifs.send(Notification(format!("{name} died!")));
            }
        }

        // Update the UI Components
        if is_us {
            for (mut text, ind_type) in &mut hp_text {
                let text_section = &mut text.sections[0].value;
                match ind_type {
                    HPIndicator::HP => {
                        // TODO no hp indication hud for now
                        *text_section = format!("");
                        //*text_section = format!("{} HP", hp.0);
                    }
                    HPIndicator::Deaths => match *total_deaths {
                        0 => {}
                        deaths => {
                            *text_section = format!("Deaths: {deaths}");
                        }
                    },
                }
            }
        }
    }
}

#[derive(Component)]
pub struct HPBar(pub Entity);

// TODO make this a child component of players and only run this on update health
fn update_hp_bar(
    players: Query<(Entity, &Health), Changed<Health>>,
    mut hp_bars: Query<(&mut Transform, &HPBar), Without<Health>>,
) {
    for (ply_ent, hp) in &players {
        info!(?hp);
        for (mut hp_bar_tfm, HPBar(owner_ent)) in hp_bars.iter_mut() {
            if &ply_ent == owner_ent {
                let health_pct = hp.0 as f32 / Health::default().0 as f32;
                hp_bar_tfm.scale = Vec3::new(0.025, health_pct / 4.0, 0.025);
            }
        }
    }
}
