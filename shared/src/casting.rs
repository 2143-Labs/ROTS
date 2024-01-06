use crate::AnyUnit;

use super::event::{spells::ShootingData, NetEntId};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct DespawnTime(pub Timer);

#[derive(Component, Debug)]
pub struct CasterNetId(pub NetEntId);

#[derive(Component)]
pub struct TargetedBullet(pub Vec3, pub NetEntId);

pub struct SharedCastingPlugin;

impl Plugin for SharedCastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_casts, update_despawns, update_casts_targeted_bullet));
    }
}

fn update_despawns(
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

fn update_casts(mut bullets: Query<(&mut Transform, &ShootingData, &DespawnTime)>) {
    for (mut bullet_tfm, shot_data, despawn_timer) in &mut bullets {
        // normalized direction
        let offset = (shot_data.target - shot_data.shot_from).normalize();
        // speed up the bullets
        let offset = offset * despawn_timer.0.elapsed_secs() * 50.0;

        let new_bullet_loc = shot_data.shot_from + offset;
        bullet_tfm.translation = new_bullet_loc;
    }
}

fn update_casts_targeted_bullet(mut bullets: Query<(&mut Transform, &TargetedBullet, &DespawnTime)>, ents: Query<(&Transform, &NetEntId), (With<AnyUnit>, Without<TargetedBullet>)>) {
    for (mut bullet_tfm, targeted_ent_data, despawn_timer) in &mut bullets {
        for (ent_tfm, ent_id) in &ents {
            if &targeted_ent_data.1 == ent_id {
                let source_location = targeted_ent_data.0;
                let target_location = ent_tfm.translation;
                // bullets take exactly 1 second
                let pct_time = despawn_timer.0.elapsed_secs();
                let distance = (target_location - source_location) * pct_time;
                let y_cord = 20.0 * pct_time - pct_time * pct_time * 20.0;
                let distance = distance + Vec3::Y * y_cord;

                let new_bullet_loc = source_location + distance;
                bullet_tfm.translation = new_bullet_loc;
            }
        }
    }
}
