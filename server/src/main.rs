use bevy_log::LogPlugin;
//use bevy::{TaskPoolPlugin, TypeRegistrationPlugin, FrameCountPlugin, HierarchyPlugin};
use bevy_time::TimePlugin;
use bevy_transform::TransformPlugin;
use bevy_hierarchy::HierarchyPlugin;
//::diagnostic::DiagnosticsPlugin;
use bevy_app::App;
use bevy_core::TaskPoolPlugin;
use bevy_core::TypeRegistrationPlugin;
use bevy_diagnostic::DiagnosticsPlugin;

fn main() {
    let mut app = App::new();

    app
        .add_plugin(LogPlugin::default())
        .add_plugin(TaskPoolPlugin::default())
        .add_plugin(TypeRegistrationPlugin::default())
        .add_plugin(TimePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(HierarchyPlugin::default())
        .add_plugin(DiagnosticsPlugin::default());

    app.run();
}
