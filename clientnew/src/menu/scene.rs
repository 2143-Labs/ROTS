use bevy::prelude::*;

use crate::states::GameState;

use super::MenuItem;

pub fn menu_select(
    keyboard_input: Res<Input<KeyCode>>,
    _config: Res<shared::Config>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.pressed(KeyCode::H) {
        game_state.set(GameState::InGame);
    }
}

#[derive(Component)]
pub enum MenuButton {
    Connect,
    Quit,
}

impl MenuButton {
    fn spawn(
        self,
        transform: Transform,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ){
        commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: 0.5,
                    })),
                    material: materials.add(Color::hex("#3090b0").unwrap().into()),
                    transform,
                    ..default()
                },
                self,
                MenuItem,
            ));
    }
}

pub fn spawn_menu_scene(
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
            MenuItem,
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
        MenuItem,
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
            MenuItem,
        ))
        .with_children(|commands| {
            commands.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(-5., 0., -5.)),
                //Collider::cuboid(5., 1.0, 6.),
            ));
        });


    MenuButton::Quit.spawn(Transform::from_xyz(3.0, 1.0, 0.0), &mut commands, &mut materials, &mut meshes);
    MenuButton::Connect.spawn(Transform::from_xyz(0.0, 1.0, 3.0), &mut commands, &mut materials, &mut meshes);

    commands.spawn((
        TextBundle::from_section(
            "test ui element",
            TextStyle {
                font: asset_server.load("fonts/fonts/ttf/JetBrainsMono-Regular.ttf"),
                font_size: 14.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        }),
        MenuItem,
    ));

}
