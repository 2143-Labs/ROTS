use bevy::prelude::*;
use crate::{AnyPlayer, event::NetEntId};

use super::event::{spells::ShootingData};

#[derive(Component)]
pub struct DespawnTime(pub Timer);

pub struct SharedCastingPlugin;

impl Plugin for SharedCastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BulletHit>()
            .add_systems(Update, (
                update_casts,
                update_despawns,
                hit,
            ));
    }
}

pub fn update_despawns(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut despawn_timer: Query<(Entity, &mut DespawnTime)>,
) {
    for (ent, mut timer) in &mut despawn_timer {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(ent).despawn_recursive();
            return;
        }
    }
}

pub fn update_casts(
    mut bullets: Query<(&mut Transform, &ShootingData, &DespawnTime)>,
) {
    for (mut bullet_tfm, shot_data, despawn_timer) in &mut bullets {

        // normalized direction
        let offset = (shot_data.target - shot_data.shot_from).normalize();
        // speed up the bullets
        let offset = offset * despawn_timer.0.elapsed_secs() * 10.0;

        let new_bullet_loc = shot_data.shot_from + offset;
        bullet_tfm.translation = new_bullet_loc;
    }
}

#[derive(Event, Debug)]
struct BulletHit {
    bullet: NetEntId,
    player: NetEntId,
}

pub fn check_collision(
    bullets: Query<&Transform, (With<ShootingData>, Without<AnyPlayer>)>,
    players: Query<&Transform, With<AnyPlayer>>,
    mut ev_w: EventWriter<BulletHit>
) {
    for bullet in &bullets {
        for player in &players {
            if bullet.translation.distance_squared(player.translation) < 1.0 {
                ev_w.send(BulletHit);
            }
        }
    }
}

pub fn hit(
    mut ev_r: EventReader<BulletHit>
) {
    for e in ev_r.read() {
        info!(?e);
    }
}
