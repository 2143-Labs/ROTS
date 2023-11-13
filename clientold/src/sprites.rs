use crate::{player::FaceCamera, states::GameState};
use bevy::prelude::*;
use bevy_sprite3d::AtlasSprite3dComponent;

pub fn init(app: &mut App) -> &mut App {
    app.add_systems(
        Update,
        (animate_sprite, face_sprite_to_camera).distributive_run_if(in_state(GameState::Ready)),
    )
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn face_sprite_to_camera(
    cam_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>,
) {
    let cam_transform = cam_query.single();
    for mut transform in query.iter_mut() {
        let current_y = transform.translation.y;
        let mut delta = cam_transform.translation - transform.translation;
        delta.y = 0.0;
        delta += transform.translation;
        transform.look_at(delta * Vec3::new(1., current_y, 1.), Vec3::Y);
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut AtlasSprite3dComponent)>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % sprite.atlas.len();
        }
    }
}
