use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{Cursor, CursorGrabMode},
};
use cameras::spawn_player_sprite;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

pub mod cameras;

fn main() {
    let mut app = App::new();

    let mut cursor = Cursor::default();
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;

    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Realm of the OctoSurvivors!".into(),
            resolution: (WIDTH, HEIGHT).into(),
            // present_mode: window::PresentMode::AutoVsync,
            resizable: false,
            // Tells wasm to resize the window according to the available canvas
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            cursor,
            ..default()
        }),
        ..default()
    };

    app
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        .add_plugins((
            FrameTimeDiagnosticsPlugin,
        ))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins((
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
            cameras::CameraPlugin,
            shared::ConfigPlugin,
        ))
        .add_systems(Startup, (spawn_scene, spawn_player_sprite))
        .run();
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let size = 30.;
    // Ground
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: size * 2.0,
                    subdivisions: 10,
                })),
                material: materials.add(Color::hex("#1f7840").unwrap().into()),
                transform: Transform::from_xyz(0.0, -0.01, 0.0),
                ..default()
            },
            Name::new("Plane"),
        ));
        //.with_children(|commands| {
            //commands.spawn((
                //Collider::cuboid(size, 1., size),
                //Name::new("PlaneCollider"),
                //TransformBundle::from(Transform::from_xyz(0., -1., 0.)),
            //));
        //});
    // Sun
    commands.spawn((
        DirectionalLightBundle {
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
        },
        Name::new("Sun"),
    ));
    // House
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("sprytilebrickhouse.gltf#Scene0"),
                transform: Transform::from_xyz(-5.2, -1.0, -20.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                ..default()
            },
            Name::new("House"),
        ))
        .with_children(|commands| {
            commands.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(-5., 0., -5.)),
                //Collider::cuboid(5., 1.0, 6.),
            ));
        });
}
