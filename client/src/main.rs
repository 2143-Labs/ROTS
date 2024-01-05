pub mod cameras;
pub mod menu;
pub mod physics;
pub mod player;
pub mod skills;
pub mod states;
pub mod worldgen;


#[cfg(feature = "mio-net")]
pub mod network;

#[cfg(not(feature = "mio-net"))]
pub mod network {
    use bevy::prelude::*;
    pub struct NetworkingPlugin;
    impl Plugin for NetworkingPlugin {
        fn build(&self, app: &mut App) {

        }
    }
}

mod cli;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{Cursor, CursorGrabMode},
};
use clap::Parser;
use shared::Config;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

fn main() {
    let args = cli::CliArgs::parse();

    if args.print_binds {
        println!("{:?}", Config::load_from_main_dir().keybindings);
        return;
    }

    if args.print_config {
        println!("{}", Config::default_config_str());
        return;
    }

    let mut app = App::new();

    let mut cursor = Cursor::default();
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;

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

    app.insert_resource(args)
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
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
            // physics::PhysPlugin,
            skills::SkillsPlugin,
            network::NetworkingPlugin,
            worldgen::WorldGenPlugin,
        ))
        .add_systems(Update, bevy::window::close_on_esc); // Close the window when you press escape

    add_inspector(&mut app);

    app.run();
}

#[cfg(feature = "inspector")]
fn add_inspector(app: &mut App) {
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
}

#[cfg(not(feature = "inspector"))]
fn add_inspector(_: &mut App) {}

pub fn despawn_all_component<T: Component>(items: Query<Entity, With<T>>, mut commands: Commands) {
    for item in &items {
        commands.entity(item).despawn_recursive();
    }
}
