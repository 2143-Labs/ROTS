use super::event::{spells::ShootingData, NetEntId};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct DespawnTime(pub Timer);

#[derive(Component, Debug)]
pub struct CasterNetId(pub NetEntId);

pub struct SharedCastingPlugin;

impl Plugin for SharedCastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_casts, update_despawns));
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
        let offset = offset * despawn_timer.0.elapsed_secs() * 10.0;

        let new_bullet_loc = shot_data.shot_from + offset;
        bullet_tfm.translation = new_bullet_loc;
    }
}
