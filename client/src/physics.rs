use bevy::prelude::*;
use bevy_xpbd_3d::prelude::PhysicsPlugins;

#[derive(Reflect, Component)]
pub struct Jumper {
    //pub cooldown: f32,
    pub timer: Timer,
}

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .register_type::<Jumper>();
    }
}
