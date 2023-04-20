use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::render_resource::Face,
    utils::tracing::instrument::WithSubscriber,
    window::{self, CursorGrabMode},
};
use bevy_asset_loader::prelude::*;
use bevy_fly_camera::{FlyCameraPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub mod player;
pub mod setup;
pub mod sprites;
pub mod states;
pub mod lifetime;
use bevy_sprite3d::Sprite3dPlugin;
use player::*;
use setup::*;
use sprites::*;
use states::*;
use lifetime::*;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;
pub const PI: f32 = 3.1415926536897932;

fn main() {
    App::new()
        // bevy_sprite3d
        .add_state::<GameState>()
        .add_state::<FreeCamState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Ready),
        )
        .add_plugin(Sprite3dPlugin)
        .add_collection_to_loading_state::<_, PlayerSpriteAssets>(GameState::Loading)
        // Background Color
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        // Load Assets
        .add_plugin(FlyCameraPlugin)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Realm of the OctoSurvivors!".into(),
                        resolution: (1280., 720.).into(),
                        // present_mode: window::PresentMode::AutoVsync,
                        resizable: false,
                        // Tells wasm to resize the window according to the available canvas
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_systems((spawn_camera, spawn_scene, spawn_tower))
        .add_system(spawn_player_sprite.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_systems(
            (player_movement, camera_follow_system)
            .distributive_run_if(in_state(FreeCamState::Locked)))
        .add_systems(
            (animate_sprite, face_sprite_to_camera)
                .distributive_run_if((in_state(GameState::Ready))),
        )
        .add_system(toggle_freecam)
        .register_type::<Tower>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        //.add_system(tower_shooting)
        .add_system(lifetime_despawn)
        // run `setup` every frame while loading. Once it detects the right
        // conditions it'll switch to the next state.
        .run()
}

fn spawn_tower(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.5, 4., 0.5))),
            material: materials.add(Color::hex("#FF0000").unwrap().into()),
            transform: Transform::from_xyz(-4., 2., 4.),
            ..default()
        })
        .insert(Tower {
            shooting_timer: Timer::from_seconds(1., TimerMode::Repeating),
        })
        .insert(Name::new("Tower"));
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
}


fn tower_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut towers: Query<&mut Tower>,
    time: Res<Time>,
) {
    for mut tower in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let spawn_transform: Transform =
                Transform::from_xyz(2., 2., 2.).with_rotation(Quat::from_rotation_y(-PI / 2.));
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube::new(0.4))),
                    material: materials.add(Color::AZURE.into()),
                    transform: spawn_transform,
                    ..default()
                })
                .insert(Lifetime {
                    timer: Timer::from_seconds(0.4, TimerMode::Once),
                })
                .insert(Name::new("Bullet"));
        }
    }
}