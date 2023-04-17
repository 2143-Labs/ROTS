use bevy::{prelude::{*}, window, diagnostic::FrameTimeDiagnosticsPlugin}; 
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use sprite::{Sprite3dPlugin, AtlasSprite3d};
mod sprite;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;
pub const PI: f32 = 3.1415926536897932;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        // .add_startup_system(Window{
        // })
        //.insert_resource(ImageSettings::default_nearest())
        
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_tower)
        .add_startup_system(spawn_player)
        // Load Assets
        .add_plugin(Sprite3dPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Realm of the OctoSurvivors!".into(),
                resolution: (1280.,720.).into(),
                present_mode: window::PresentMode::AutoVsync,
                resizable: false,
                // Tells wasm to resize the window according to the available canvas
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .register_type::<Tower>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugins(DefaultPlugins)
        .add_system(tower_shooting)
        .add_system(lifetime_despawn)
        // run `setup` every frame while loading. Once it detects the right
        // conditions it'll switch to the next state.
        .run()
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let plane = commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10. , subdivisions: 1 })),
        material: materials.add(Color::hex("#ff00ff").unwrap().into()),
        ..default()
    }).insert(Name::new("Plane"));
    let cube = commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.})),
        material: materials.add(Color::hex("#FFFFFF").unwrap().into()),
        transform: Transform::from_xyz(0.,0.5,0.),
        ..default()
    }).insert(Name::new("Cube"));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.,8.,4.),
        ..default()
    }).insert(Name::new("Sun"));
}
fn spawn_tower(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
){
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.5,4.,0.5))),
        material: materials.add(Color::hex("#FF0000").unwrap().into()),
        transform: Transform::from_xyz(-4.,2.,4.),
        ..default()
    })
    .insert(Tower {
        shooting_timer: Timer::from_seconds(1.,TimerMode::Repeating)
    })
    .insert(Name::new("Tower"));
}

fn spawn_player(
    mut commands: Commands,
    mut sprite_params: sprite::Sprite3dParams,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
){
    let texture_handle = asset_server.load("MrMan.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.,44.),2, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn(AtlasSprite3d {
        atlas: texture_atlas_handle,
        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,
        index: 1, 
        ..default()
    }.bundle(&mut sprite_params))
    .insert(Name::new("Player"));
}
pub struct Player {
    position: Vec3,
    velocity: Vec3,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);


#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime{
    timer: Timer,
}

fn tower_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut towers: Query<&mut Tower>,
    time: Res<Time>,
){
    for mut tower in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
           let spawn_transform: Transform = 
            Transform::from_xyz(2., 2., 2.)
            .with_rotation(Quat::from_rotation_y(-PI / 2.));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(0.4))),
                material: materials.add(Color::AZURE.into()),
                transform: spawn_transform,
                ..default()
            })
            .insert(Lifetime {
                timer: Timer::from_seconds(0.4, TimerMode::Once)
            })
            .insert(Name::new("Bullet"));
        }
    }
}
fn lifetime_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}