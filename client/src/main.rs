pub mod cameras;
pub mod menu;
pub mod network;
pub mod physics;
pub mod player;
pub mod skills;
pub mod states;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{Cursor, CursorGrabMode},
};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

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

    app.insert_resource(ClearColor(Color::hex("212121").unwrap()))
        .add_plugins((FrameTimeDiagnosticsPlugin,))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins((
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
            cameras::CameraPlugin,
            cameras::notifications::NotificationPlugin,
            shared::ConfigPlugin,
            states::StatePlugin,
            menu::MenuPlugin,
            physics::PhysPlugin,
            skills::SkillsPlugin,
            network::NetworkingPlugin,
        ))
        .add_systems(Update, bevy::window::close_on_esc) // Close the window when you press escape
        .run();
}

pub fn despawn_all_component<T: Component>(items: Query<Entity, With<T>>, mut commands: Commands) {
    for item in &items {
        commands.entity(item).despawn_recursive();
    }
}
