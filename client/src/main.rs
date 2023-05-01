use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{Cursor, CursorGrabMode},
};
use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_sprite3d::Sprite3dPlugin;
use networking::client_bullet_receiver::NetworkingPlugin;
use states::StatePlugin;

pub mod camera;
pub mod lifetime;
pub mod networking;
pub mod physics;
pub mod player;
pub mod setup;
pub mod sprites;
pub mod states;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

fn main() {
    let mut app = App::new();

    setup::init(&mut app);
    camera::init(&mut app);
    player::init(&mut app);
    lifetime::init(&mut app);
    sprites::init(&mut app);
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
        // bevy_sprite3d
        .add_plugin(Sprite3dPlugin)
        // Background Color
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        // Load Assets
        .add_plugin(FlyCameraPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins(
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            ..default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(StatePlugin)
        .add_plugin(NetworkingPlugin)
        .run();
}
