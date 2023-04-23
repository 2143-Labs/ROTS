use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};
use bevy_mod_raycast::{RaycastSource, RaycastMesh, DefaultRaycastingPlugin};

use crate::{player::{FaceCamera}, sprites::AnimationTimer, states::GameState};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_startup_systems((spawn_camera, spawn_scene))
        .add_system(spawn_muscle_man.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
}

#[derive(Component)]
struct PlayerCamera; // tag entity to make it always face the camera

#[derive(Reflect, Component)]
pub struct CameraFollow {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
        }
    }
}

#[derive(Reflect, Clone)]
pub struct MyRaycastSet;

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            RaycastSource::<MyRaycastSet>::new_transform_empty()
        ))
        .insert(CameraFollow::default())
        .insert(Name::new("Camera"))
        .insert(PlayerCamera);
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: 100.,
                    subdivisions: 10,
                })),
                material: materials.add(Color::hex("#1f7840").unwrap().into()),
                ..default()
            },
            RaycastMesh::<MyRaycastSet>::default(),
        ))
        .insert(Name::new("Plane"));

    commands
        .spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            ))
            .mul_transform(Transform::from_rotation(Quat::from_rotation_y(
                -std::f32::consts::FRAC_PI_4,
            ))),
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 0.9, 0.8),
                illuminance: 15_000.0,
                shadows_enabled: true,
                ..default()
            },
            ..Default::default()
        })
        .insert(Name::new("Sun"));
}

#[derive(AssetCollection, Resource)]
pub struct MuscleManAssets{
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64.))]
    #[asset(texture_atlas(columns = 21, rows = 1))]
    #[asset(path = "buff-Sheet.png")]
    pub run: Handle<TextureAtlas>,
}
pub fn spawn_muscle_man(
    mut commands: Commands,
    images: Res<MuscleManAssets>,
    mut sprite_params: Sprite3dParams,
) {
    let sprite = AtlasSprite3d {
        atlas: images.run.clone(),

        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,

        index: 1,

        transform: Transform::from_xyz(0., 1., 0.),
        // pivot: Some(Vec2::new(0.5, 0.5)),
        ..default()
    }
    .bundle(&mut sprite_params);

    commands
        .spawn(sprite)
        .insert(FaceCamera)
        .insert(AnimationTimer(Timer::from_seconds(
            0.2,
            TimerMode::Repeating,
        )));
}
