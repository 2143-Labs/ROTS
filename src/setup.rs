use bevy::prelude::*;

#[derive(Component)]
struct PlayerCamera; // tag entity to make it always face the camera

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
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
            material: materials.add(Color::hex("#ff00ff").unwrap().into()),
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
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4., 8., 4.),
            ..default()
        })
        .insert(Name::new("Sun"));
}
