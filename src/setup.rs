use bevy::prelude::*;

pub fn init(app: &mut App) -> &mut App {
    app.add_startup_systems((spawn_camera, spawn_scene))
}

#[derive(Component)]
struct PlayerCamera; // tag entity to make it always face the camera

#[derive(Reflect, Component)]
pub struct CameraFollow {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraFollow::default())
        .insert(Name::new("Camera"))
        .insert(PlayerCamera);
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 10.,
                subdivisions: 1,
            })),
            material: materials.add(Color::hex("#1f7840").unwrap().into()),
            ..default()
        })
        .insert(Name::new("Plane"));
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1. })),
            material: materials.add(Color::hex("#FFFFFF").unwrap().into()),
            transform: Transform::from_xyz(0., 0.5, 0.),
            ..default()
        })
        .insert(Name::new("Cube"));

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
}
