use std::f32::consts::PI;

use bevy::{prelude::*, render::{camera::RenderTarget, render_resource::{TextureDimension, TextureDescriptor, TextureFormat, TextureUsages, Extent3d}}, ecs::query::QuerySingleError};

use crate::{states::GameState, cameras::Player};

use super::MenuItem;

pub fn menu_select(
    keyboard_input: Res<Input<KeyCode>>,
    _config: Res<shared::Config>,
    mut game_state: ResMut<NextState<GameState>>,
    buttons: Query<&MenuButton, With<SelectedButton>>,
) {

    if !keyboard_input.just_pressed(KeyCode::H) {
        return;
    }

    let button = match buttons.get_single() {
        Ok(button) => button,
        Err(QuerySingleError::NoEntities(_)) => {
            // Play sound error
            info!("Not near a button");
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            info!("Somehow got multiple selected buttons?");
            return;
        }
    };

    info!("Clicked button {button:?}");
    game_state.set(GameState::InGame);
}

#[derive(Component, Debug)]
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

#[derive(Component)]
pub struct SelectedButton;

pub fn check_is_next_to_button(
    mut commands: Commands,
    players: Query<(Entity, &Player, &Transform)>,
    mut buttons: Query<(Entity, &MenuButton, &mut Transform, Option<&SelectedButton>), Without<Player>>,
    time: Res<Time>
) {
    // This system will add or remove the `SelectedButton` component, and make the buttons spin
    let max_dist = 2.0;
    for (_player_ent, _player, player_transform) in &players {
        for (button_ent, _button, mut button_transform, selected) in &mut buttons {
            if selected.is_some() {
                button_transform.rotate_x(time.delta_seconds() * 1.5);
                button_transform.rotate_y(time.delta_seconds() * 3.0);
                button_transform.rotate_z(time.delta_seconds() * 6.0);
                let dist = player_transform.translation.distance(button_transform.translation);
                if dist > max_dist {
                    info!("Left range of {button_ent:?}");
                    commands.entity(button_ent).remove::<SelectedButton>();
                }
            } else {
                button_transform.rotation = Quat::from_rotation_y(0.0);
                let dist = player_transform.translation.distance(button_transform.translation);
                if dist < max_dist {
                    info!("Approached {button_ent:?}");
                    commands.entity(button_ent).insert(SelectedButton);
                }
            }
        }
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

}

fn _test_sub_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    commands.spawn((
        TextBundle::from_section(
            "GameState: MASDFASDF",
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


    let cam_size = Extent3d {
        width: 1000,
        height: 1000,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: cam_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(cam_size);

    let image_handle = images.add(image);

    let camera = Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        ..default()
    };

    commands.spawn((
        camera,
        MenuItem,
    ));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    let cube_size = 1.0;
    let cube_handle = meshes.add(Mesh::from(shape::Plane { size: cube_size, subdivisions: 4 }));

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform::from_xyz(-3.0, 1.0, 0.0)
                .with_rotation(Quat::from_rotation_x(PI/2.0))
                .with_rotation(Quat::from_rotation_y(PI/2.0)),
            ..default()
        },
        MenuItem,
    ));
}
