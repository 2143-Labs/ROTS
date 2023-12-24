use bevy::prelude::*;
use bevy_xpbd_3d::prelude::PhysicsPlugins;

#[derive(Reflect, Component)]
pub struct Jumper {
    //pub cooldown: f32,
    pub timer: Timer,
}

impl Jumper {
    pub fn get_y(&self) -> f32 {
        if self.timer.finished() {
            return 0.0;
        }

        let delta = self.timer.elapsed_secs();
        let x = -delta * delta * 9.8 + 10.0 * delta;
        x.max(0.0)
    }
}

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .register_type::<Jumper>();
    }
}
