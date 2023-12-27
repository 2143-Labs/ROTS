use bevy::{ecs::world, prelude::*, render::mesh::shape::Plane};
use bevy_xpbd_3d::components::Collider;
use noise::{NoiseFn, Perlin, Seedable};
use std::collections::HashMap;

use crate::{cli::CliArgs, player::Player};
const CHUNK_SIZE: i32 = 16;
const TREE_HEIGHT: f32 = 8.0;

pub struct Chunk {
    pub position: Vec2,
    pub size: f32,
    pub tree_density: f32,
}
#[derive(PartialEq, Eq, Hash, Reflect, Clone)]
pub struct ChunkPos(pub i32, pub i32, pub i32);

#[derive(Resource)]
pub struct World {
    // TODO make these entities with a ChunkPos component
    chunks: HashMap<ChunkPos, Entity>,
    noise: Perlin,
}

#[derive(Resource, Clone)]
pub struct WorldMaterialAssets {
    ground_mesh: Handle<Mesh>,
    material_grass_dark: Handle<StandardMaterial>,
    material_grass: Handle<StandardMaterial>,
    tree_mesh: Handle<Mesh>,
    material_tree: Handle<StandardMaterial>,
}

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World {
            chunks: HashMap::new(),
            noise: Perlin::new(421018),
        })
        .insert_resource(WorldMaterialAssets {
            ground_mesh: Handle::default(),
            material_grass_dark: Handle::default(),
            material_grass: Handle::default(),
            tree_mesh: Handle::default(),
            material_tree: Handle::default(),
        })
        .add_systems(Update, update_chunks.run_if(should_run_update))
        .add_systems(Startup, init_mats);
    }
}

fn init_mats(
    mut world: ResMut<WorldMaterialAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    args: Res<CliArgs>,
) {
    world.material_grass = materials.add(Color::rgb(0.06, 0.687, 0.238).into());
    world.material_grass_dark = materials.add(Color::rgb(0.04, 0.48, 0.164).into());
    world.ground_mesh = meshes.add(Mesh::from(shape::Plane {
        size: CHUNK_SIZE as f32,
        subdivisions: 0,
    }));

    world.tree_mesh = meshes.add(Mesh::from(shape::Box::new(0.25, TREE_HEIGHT, 0.25)));
    world.material_tree = materials.add(Color::rgb(0.4, 0.2, 0.0).into());

    if args.optimize_floor() {
        commands.spawn((PbrBundle {
            transform: Transform::from_xyz(0.0, -0.01, 0.0),
            mesh: meshes.add(Mesh::from(Plane {
                size: (CHUNK_SIZE * 100) as f32,
                subdivisions: 1,
            })),
            material: materials.add(Color::hex("#1f7840").unwrap().into()),
            ..Default::default()
        },));
    }
}

// This is an optimization in debug mode
fn should_run_update(args: Res<CliArgs>) -> bool {
    !args.optimize_floor()
}

fn spawn_chunk_objects(
    chunk_pos: ChunkPos,
    commands: &mut Commands,
    world_assets: WorldMaterialAssets,
    noise: &Perlin,
    chunk: Entity,
) {
    let tree_width = CHUNK_SIZE / 4;
    for x in 0..tree_width {
        for y in 0..tree_width {
            _ = spawn_tree_in_tile(
                chunk,
                commands,
                &chunk_pos,
                Vec2::new((x * CHUNK_SIZE / 4) as f32, (y * CHUNK_SIZE / 4) as f32),
                noise,
                world_assets.tree_mesh.clone(),
                world_assets.material_tree.clone(),
            )
        }
    }
}
fn spawn_tree_in_tile(
    chunk: Entity,
    commands: &mut Commands,
    chunk_pos: &ChunkPos,
    tile_pos: Vec2,
    noise: &Perlin,
    tree_mesh: Handle<Mesh>,
    tree_material: Handle<StandardMaterial>,
) {
    let scale = 0.7;
    let point = [
        (chunk_pos.0 as f32 + tile_pos.x) as f64 * scale,
        (chunk_pos.2 as f32 + tile_pos.y) as f64 * scale,
    ];
    let tree_density = noise.get(point) + 1. / 2.;
    let tree_width = CHUNK_SIZE / 4;
    println!("noise: {}, point {},{}", tree_density, point[0], point[1]);

    // If the tree density is above a certain threshold, spawn a tree in this tile
    // print the noise value for debugging
    if tree_density > 0.75 {
        // Spawn a tree entity as a child of the chunk
        commands.entity(chunk).with_children(|parent| {
            parent.spawn((
                // You'll need to implement this yourself
                PbrBundle {
                    mesh: tree_mesh,
                    material: tree_material,
                    transform: Transform::from_translation(Vec3::new(
                        -(CHUNK_SIZE / 2) as f32 + tile_pos.x + 0.5 + tree_width as f32 / 2.0,
                        TREE_HEIGHT / 2 as f32,
                        -(CHUNK_SIZE / 2) as f32 + tile_pos.y + 0.5 + tree_width as f32 / 2.0,
                    )),
                    ..Default::default()
                },
                Name::new(format!("Tree: {}", tile_pos)),
            ));
        });
    }
}

fn update_chunks(
    mut commands: Commands,
    mut player_query: Query<(&Transform, Entity, &mut Player), Changed<Transform>>,
    mut world: ResMut<World>,
    world_assets: Res<WorldMaterialAssets>,
) {
    for (transform, _player_ent, mut _player) in player_query.iter_mut() {
        let view_distance = 5;
        let player_chunk_pos_vec = transform.translation / CHUNK_SIZE as f32;
        let player_chunk_pos = ChunkPos(
            player_chunk_pos_vec.x.floor() as i32,
            player_chunk_pos_vec.y.floor() as i32,
            player_chunk_pos_vec.z.floor() as i32,
        );
        if _player.current_chunk == player_chunk_pos {
            continue;
        };
        //info!("Recalculating chunks");
        _player.current_chunk = player_chunk_pos;
        // let player_chunk_pos = transform.translation().truncate() / CHUNK_SIZE;
        let player_chunk_pos = Vec3::new(
            player_chunk_pos_vec.x.floor(),
            player_chunk_pos_vec.y.floor(),
            player_chunk_pos_vec.z.floor(),
        );

        // Calculate the range of chunks that should be visible
        let min = player_chunk_pos - Vec3::splat(view_distance as f32);
        let max = player_chunk_pos + Vec3::splat(view_distance as f32);

        // Spawn new chunks
        for x in (min.x as i32)..=(max.x as i32) {
            for z in (min.z as i32)..=(max.z as i32) {
                // Calculate the distance from this chunk to the player's chunk
                let distance = ((x as f32).powi(2) + (z as f32).powi(2)).sqrt()
                    - (player_chunk_pos.x.powi(2) + player_chunk_pos.z.powi(2)).sqrt();

                // Only spawn the chunk if it's within the view distance
                if distance <= view_distance as f32 {
                    let chunk_pos = ChunkPos(x, 0, z);
                    if !world.chunks.contains_key(&chunk_pos) {
                        let chunk_pos_vec3 = Vec3::new(chunk_pos.0 as f32, 0., chunk_pos.2 as f32);
                        let material = if (chunk_pos.0 + chunk_pos.2) % 2 == 0 {
                            world_assets.material_grass.clone()
                        } else {
                            world_assets.material_grass_dark.clone()
                        };
                        let chunk = commands
                            .spawn((
                                PbrBundle {
                                    mesh: world_assets.ground_mesh.clone(),
                                    // random color for chunks
                                    material: material,
                                    transform: Transform::from_translation(
                                        chunk_pos_vec3 * CHUNK_SIZE as f32
                                            + Vec3::new(
                                                CHUNK_SIZE as f32 / 2.,
                                                0.,
                                                CHUNK_SIZE as f32 / 2.,
                                            ),
                                    ),
                                    ..Default::default()
                                },
                                Collider::cuboid(CHUNK_SIZE as f32, 0.002, CHUNK_SIZE as f32),
                                Name::new(format!("Chunk: {}", chunk_pos_vec3)),
                            ))
                            .id();
                        world.chunks.insert(chunk_pos.clone(), chunk);
                        spawn_chunk_objects(
                            chunk_pos,
                            &mut commands,
                            world_assets.clone(),
                            &world.noise,
                            chunk,
                        );
                    }
                }
            }
        }
        // Despawn old chunks
        world.chunks.retain(|pos, entity| {
            // Calculate the distance from this chunk to the player's chunk
            let dx = pos.0 as f32 - player_chunk_pos.x as f32;
            let dz = pos.2 as f32 - player_chunk_pos.z as f32;
            let distance = (dx.powi(2) + dz.powi(2)).sqrt();

            if distance <= view_distance as f32 {
                true
            } else {
                commands.entity(*entity).despawn_recursive();
                false
            }
        });
    }
}
