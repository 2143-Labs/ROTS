use bevy::{prelude::{*}, window, diagnostic::FrameTimeDiagnosticsPlugin}; 
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_sprite3d::{Sprite3dPlugin, AtlasSprite3d, Sprite3dParams, AtlasSprite3dComponent};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;
pub const PI: f32 = 3.1415926536897932;



#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
enum GameState { 
    #[default]
    Loading, 
    Ready
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    #[asset(texture_atlas(columns = 3, rows = 1))]
    #[asset(path = "brownSheet.png")]
    pub run: Handle<TextureAtlas>,
}

fn main() {
    App::new()
        // bevy_sprite3d
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
        )
        .add_plugin(Sprite3dPlugin)
        .add_collection_to_loading_state::<_,ImageAssets>(GameState::Loading)


        // Background Color
        .insert_resource(ClearColor(Color::hex("212121").unwrap()))
        
        // Load Assets
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
            })
            .set(ImagePlugin::default_nearest())
        )
        .add_startup_systems((spawn_camera, spawn_scene, spawn_tower))
        .add_system(spawn_player_sprite.run_if(in_state(GameState::Ready).and_then(run_once())))
        .add_system(animate_sprite.run_if((in_state(GameState::Ready))))
        .register_type::<Tower>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        //.add_system(tower_shooting)
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10. , subdivisions: 1 })),
        material: materials.add(Color::hex("#ff00ff").unwrap().into()),
        ..default()
    }).insert(Name::new("Plane"));
    commands.spawn(PbrBundle {
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

fn spawn_player_sprite(
    mut commands: Commands, 
    images: Res<ImageAssets>,
    mut sprite_params: Sprite3dParams
){
    commands.spawn(AtlasSprite3d {
        atlas: images.run.clone(),

        pixels_per_metre: 32.,
        partial_alpha: true,
        unlit: true,

        index: 1,

        transform: Transform::from_xyz(-3., 1., 2.).looking_at( Vec3::new(10.,10.,10.), Vec3::Y),
        // pivot: Some(Vec2::new(0.5, 0.5)),

        ..default()
    }.bundle(&mut sprite_params))
    .insert(Name::new("PlayerSprite"))
    .insert(AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)));
}
fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut AtlasSprite3dComponent)>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % sprite.atlas.len();
        }
    }
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
