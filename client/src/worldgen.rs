use bevy::{prelude::*, render::mesh::shape::Plane};
use bevy_xpbd_3d::components::Collider;
use std::collections::HashMap;

use crate::{cli::CliArgs, player::Player};
const CHUNK_SIZE: i32 = 1;

pub struct Chunk {
    pub position: Vec3,
    pub size: f32,
}
#[derive(PartialEq, Eq, Hash, Reflect)]
pub struct ChunkPos(pub i32, pub i32, pub i32);

#[derive(Resource)]
pub struct World {
    // TODO make these entities with a ChunkPos component
    chunks: HashMap<ChunkPos, Entity>,
}

#[derive(Resource)]
pub struct WorldMaterialAssets {
    mesh: Handle<Mesh>,
    material_purp: Handle<StandardMaterial>,
    material_black: Handle<StandardMaterial>,
}

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World {
            chunks: HashMap::new(),
        })
        .insert_resource(WorldMaterialAssets {
            mesh: Handle::default(),
            material_purp: Handle::default(),
            material_black: Handle::default(),
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
    world.material_black = materials.add(Color::rgb(0.0, 0.0, 0.0).into());
    world.material_purp = materials.add(Color::rgb(0.5, 0.0, 0.5).into());
    world.mesh = meshes.add(Mesh::from(shape::Plane {
        size: CHUNK_SIZE as f32,
        subdivisions: 0,
    }));

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

fn update_chunks(
    mut commands: Commands,
    mut player_query: Query<(&Transform, Entity, &mut Player), Changed<Transform>>,
    mut world: ResMut<World>,
    world_assets: Res<WorldMaterialAssets>,
) {
    for (transform, _player_ent, mut _player) in player_query.iter_mut() {
        let view_distance = 15;
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
                            world_assets.material_black.clone()
                        } else {
                            world_assets.material_purp.clone()
                        };
                        let chunk = commands
                            .spawn((
                                PbrBundle {
                                    mesh: world_assets.mesh.clone(),
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
                        world.chunks.insert(chunk_pos, chunk);
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
                commands.entity(*entity).despawn();
                false
            }
        });
    }
}
