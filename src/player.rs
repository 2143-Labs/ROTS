use crate::sprites::AnimationTimer;
use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};

#[derive(Component)]
pub struct PlayerCamera; // tag entity to make it always face the camera

#[derive(AssetCollection, Resource)]
pub struct PlayerSpriteAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    #[asset(texture_atlas(columns = 3, rows = 1))]
    #[asset(path = "brownSheet.png")]
    pub run: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct FaceCamera; // tag entity to make it always face the camera

#[derive(Reflect, Component)]
pub struct Player {
    pub looking_at: Vec3,
    pub facing_vel: f32,
    pub position: Vec3,
    pub velocity: Vec3,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn spawn_player_sprite(
    mut commands: Commands,
    images: Res<PlayerSpriteAssets>,
    mut sprite_params: Sprite3dParams,
) {
    let sprite = AtlasSprite3d {
        atlas: images.run.clone(),

        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,

        index: 1,

        transform: Transform::from_xyz(-3., 1., 2.)
            .looking_at(Vec3::new(10., 10., 10.), Vec3::Y),
        // pivot: Some(Vec2::new(0.5, 0.5)),
        ..default()
    }
    .bundle(&mut sprite_params);

    commands
        .spawn(sprite)
        .insert(Name::new("PlayerSprite"))
        .insert(Player::default())
        .insert(FaceCamera)
        .insert(AnimationTimer(Timer::from_seconds(
            0.4,
            TimerMode::Repeating,
        )));
}
