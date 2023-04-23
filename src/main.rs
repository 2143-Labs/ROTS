use std::f32::consts::PI;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_sprite3d::Sprite3dPlugin;

pub mod lifetime;
pub mod player;
pub mod setup;
pub mod sprites;
pub mod states;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

fn main() {
    let mut app = App::new();

    states::init(&mut app);
    setup::init(&mut app);
    player::init(&mut app);
    sprites::init(&mut app);
    lifetime::init(&mut app);

    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Realm of the OctoSurvivors!".into(),
            resolution: (WIDTH, HEIGHT).into(),
            // present_mode: window::PresentMode::AutoVsync,
            resizable: false,
            // Tells wasm to resize the window according to the available canvas
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    };

    app
        // bevy_sprite3d
        .add_plugin(Sprite3dPlugin)
        // Background Color
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        // Load Assets
        .add_plugin(FlyCameraPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins(
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
        )
        .register_type::<Tower>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        // TODO refactor into another system
        .add_startup_system(spawn_tower)
        //.add_system(tower_shooting)
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

fn _tower_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut towers: Query<&mut Tower>,
    time: Res<Time>,
) {
    for mut tower in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if !tower.shooting_timer.just_finished() {
            continue;
        }

        let spawn_transform: Transform =
            Transform::from_xyz(2., 2., 2.).with_rotation(Quat::from_rotation_y(-PI / 2.));

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(0.4))),
                material: materials.add(Color::AZURE.into()),
                transform: spawn_transform,
                ..default()
            })
            .insert(lifetime::Lifetime {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
            })
            .insert(Name::new("Bullet"));
    }
}
