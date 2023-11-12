use bevy::prelude::*;
use bevy_xpbd_3d::prelude::PhysicsPlugins;

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
    }
}
