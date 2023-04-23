use bevy::prelude::*;
use bevy_rapier3d::prelude::ActiveEvents;

pub fn modify_collider_active_events(mut active_events: Query<&mut ActiveEvents>) {
    for active_event in active_events.iter_mut() {
        debug!(?active_event);
    }
}
