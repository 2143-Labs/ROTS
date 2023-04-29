use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastMesh, RaycastSource};
use bevy_rapier3d::prelude::*;
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};

use crate::{
    physics::modify_collider_active_events, player::FaceCamera, sprites::AnimationTimer,
    states::GameState,
};

pub fn init(app: &mut App) -> &mut App {
    app.add_startup_systems((spawn_camera, spawn_scene))
        .add_system(modify_collider_active_events)
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
    pub dragging: bool,
    pub degrees: f32,
    pub old_degrees: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            dragging: false,
            degrees: 0.,
            old_degrees: 0.,
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
            RaycastSource::<MyRaycastSet>::new_transform_empty(),
        ))
        .insert(CameraFollow::default())
        .insert(Name::new("Camera"))
        .insert(PlayerCamera);
}

#[derive(Component)]
pub struct Hideable;

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let size = 10.;
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: size * 2.0,
                    subdivisions: 10,
                })),
                material: materials.add(Color::hex("#1f7840").unwrap().into()),
                ..default()
            },
            RaycastMesh::<MyRaycastSet>::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn(Collider::cuboid(size, 1., size))
                .insert(TransformBundle::from(Transform::from_xyz(0., -1., 0.)));
        })
        .insert(Hideable)
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
    commands.spawn(SceneBundle {
        scene: asset_server.load("sprytilebrickhouse.gltf#Scene0"),
        transform: Transform::from_xyz(-5.2,-1.0,-20.0).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ..default()
    })
    .with_children(|x| {
        x.spawn(Collider::cuboid(5., 1.0, 6.))
        .insert(TransformBundle::from(Transform::from_xyz(-5., 0., -5.)));
    })
    .insert(Name::new("House"));
}

#[derive(AssetCollection, Resource)]
pub struct MuscleManAssets {
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
