use crate::{
    sprites::AnimationTimer,
    states::GameState, camera::PlayerCamera,
};
use bevy::prelude::*;

use bevy_asset_loader::prelude::AssetCollection;
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, ExternalImpulse, GravityScale, LockedAxes, RigidBody,
};
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_system(spawn_player_sprite.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_system(player_movement.run_if(in_state(GameState::Ready)))
        .register_type::<Jumper>()
}

#[derive(Reflect, Component)]
pub struct Jumper {
    //pub cooldown: f32,
    pub timer: Timer,
}

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
    pub velocity: Vec3,
    pub lock_movement: [Option<Vec2>; 4],
}
impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            velocity: Vec3::ZERO,
            lock_movement: [None; 4],
        }
    }
}

pub fn spawn_player_sprite(
    mut commands: Commands,
    images: Res<PlayerSpriteAssets>,
    mut sprite_params: Sprite3dParams,
) {
    let starting_location = Vec3::new(-3., 0.5, 2.);
    let sprite = AtlasSprite3d {
        atlas: images.run.clone(),

        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,

        index: 1,
        // pivot: Some(Vec2::new(0.5, 0.5)),
        ..default()
    }
    .bundle(&mut sprite_params);

    commands
        .spawn(sprite)
        .insert(TransformBundle::from_transform(Transform::from_translation(starting_location)))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.2))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(GravityScale(1.))
        .insert(ColliderMassProperties::Mass(1.0))
        .insert(Name::new("PlayerSprite"))
        .insert(Player::default())
        .insert(FaceCamera)
        .insert(Jumper {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .insert(Name::new("PlayerBody"))
        .insert(AnimationTimer(Timer::from_seconds(
            0.4,
            TimerMode::Repeating,
        )));
}

pub const PLAYER_SPEED: f32 = 5.;
pub fn player_movement(
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, Entity, &mut Jumper), With<Player>>,
    camera_query: Query<&PlayerCamera>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, player_ent, mut jumper) in player_query.iter_mut() {
        let mut move_vector = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            move_vector += Vec2::new(1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_vector += Vec2::new(-1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_vector += Vec2::new(0.0, -1.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_vector += Vec2::new(0.0, 1.0);
        }

        jumper.timer.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if jumper.timer.finished() {
                commands.entity(player_ent).insert(ExternalImpulse {
                    impulse: Vec3::new(0., 4., 0.),
                    torque_impulse: Vec3::new(0., 0., 0.),
                });
                jumper.timer.reset();
            }
        }

        if move_vector.length_squared() > 0.0 {
            let camera = camera_query.single();
            let rotation = Vec2::from_angle(-camera.yaw_radians);
            let movem = move_vector.normalize().rotate(rotation);

            transform.translation += Vec3::new(movem.x, 0.0, movem.y) * PLAYER_SPEED * time.delta_seconds();
        }
    }
}
