use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_rapier3d::prelude::Collider;
use rand::Rng;

use crate::{states::GameState, player::Player, setup::Hideable};

pub fn init(app: &mut App) -> &mut App {
    app
        .insert_resource(ChunkMapResource::default())
        .add_system(gen_world.run_if(in_state(GameState::Ready)))
        .add_startup_system(init_biomes)
}

fn init_biomes(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create the game assets
    let mut world_assets = WorldAssets::default();

    // Initialize the biome materials
    world_assets.biome_materials.forest = Some(Biome::Forest.create_material(&mut materials));
    world_assets.biome_materials.desert = Some(Biome::Desert.create_material(&mut materials));
    world_assets.biome_materials.mountains = Some(Biome::Mountains.create_material(&mut materials));
    world_assets.biome_materials.ocean = Some(Biome::Ocean.create_material(&mut materials));
    world_assets.biome_materials.plains = Some(Biome::Plains.create_material(&mut materials));
    
    // Spawn the game assets as a resource
    commands.insert_resource(world_assets);
}


#[derive(Clone, Debug, PartialEq)]
struct Chunk {
    position: Vec2,
    biome: Biome,
    height_variation: f32,
    // Other chunk data...
}

impl Chunk {
    fn new(position: Vec2, biome: Biome) -> Self {
        Chunk {
            position,
            biome,
            height_variation: biome.default_height_variation(),
            // Initialize other chunk data...
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Biome {
    Forest,
    Desert,
    Mountains,
    Ocean,
    Plains,
    // Add more biome variants as needed
}

struct BiomeMaterials {
    forest: Option<Handle<StandardMaterial>>,
    desert: Option<Handle<StandardMaterial>>,
    mountains: Option<Handle<StandardMaterial>>,
    ocean: Option<Handle<StandardMaterial>>,
    plains: Option<Handle<StandardMaterial>>,
}

impl Default for BiomeMaterials {
    fn default() -> Self {
        BiomeMaterials {
            forest: None,
            desert: None,
            mountains: None,
            ocean: None,
            plains: None,
        }
    }
}

#[derive(AssetCollection, Resource)]
struct WorldAssets {
    biome_materials: BiomeMaterials,
    // Other game assets...
}

impl Default for WorldAssets {
    fn default() -> Self {
        WorldAssets {
            biome_materials: BiomeMaterials::default(),
            // Initialize other game assets...
        }
    }
}

impl Biome {
    fn create_material(&self, biome_materials: &mut ResMut<Assets<StandardMaterial>>) -> Handle<StandardMaterial> {
        match self {
            Biome::Forest => {
                let material = StandardMaterial {
                    base_color: Color::rgb(0.0, 1.0, 0.0),
                    // Set other material properties as needed
                    ..Default::default()
                };
                biome_materials.add(material)
            }
            Biome::Desert => {
                let material = StandardMaterial {
                    base_color: Color::rgb(1.0, 1.0, 0.0),
                    // Set other material properties as needed
                    ..Default::default()
                };
                biome_materials.add(material)
            }
            Biome::Mountains => {
                let material = StandardMaterial {
                    base_color: Color::rgb(0.5, 0.5, 0.5),
                    // Set other material properties as needed
                    ..Default::default()
                };
                biome_materials.add(material)
            }
            Biome::Ocean => {
                let material = StandardMaterial {
                    base_color: Color::rgb(0.0, 0.0, 1.0),
                    // Set other material properties as needed
                    ..Default::default()
                };
                biome_materials.add(material)
            }
            Biome::Plains => {
                let material = StandardMaterial {
                    base_color: Color::rgb(0.0, 0.8, 0.0),
                    // Set other material properties as needed
                    ..Default::default()
                };
                biome_materials.add(material)
            }
            // Add custom creation logic for other biome variants
        }
    }

    fn default_height_variation(&self) -> f32 {
        let max_variation = match self {
            Biome::Forest => 0.5,
            Biome::Desert => 0.3,
            Biome::Mountains => 1.0,
            Biome::Ocean => 0.2,
            Biome::Plains => 0.1,
            // Add custom default height variation for other biome variants
        };
        rand::thread_rng().gen_range(-max_variation..max_variation)
    }
}

#[derive(AssetCollection, Resource)]
struct ChunkMapResource {
    chunk_map: HashMap<Vec2, Chunk>,
    biome_list: Vec<(Vec2, Biome)>,
}

impl Default for ChunkMapResource {
    fn default() -> Self {
        ChunkMapResource {
            chunk_map: HashMap::default(),
            biome_list: vec![(
                Vec2::new(0., 0.),
                Biome::Forest,
            ),
            (
                Vec2::new(10., 30.),
                Biome::Desert,
            ),
            (
                Vec2::new(-20., -5.),
                Biome::Mountains,
            ),
            (
                Vec2::new(-6.,30.),
                Biome::Ocean,
            ),
            (
                Vec2::new(21.0, -40.0),
                Biome::Plains,
            )],
        }   
    }
}

pub fn spawn_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block_id: [i32; 2]
) {
    let size = 5.;
    let odd_block = (block_id[0] + block_id[1]).abs() %2i32 == 1;
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(size, 2., size))),
                material: materials.add(
                    Color::hex(
                        if odd_block { "#000000" } else { "#ff00ff" }
                    ).unwrap().into()),
                ..default()
            },
            Collider::cuboid(size/2., 1., size/2.))
        )
        .insert(TransformBundle::from(Transform::from_xyz(
            block_id[0] as f32 * size,
            -1.,
            block_id[1] as f32 * size
        )))
        .insert(Hideable)
        .insert(Name::new(format!("Block_{}_{}", block_id[0], block_id[1])));
}

pub fn gen_world(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    for (transform, mut player) in player_query.iter_mut() {
        let standing_on_block_id: [i32; 2] = [
            ((5. + transform.translation.abs().x - 2.5) * (transform.translation.signum().x)) as i32 / 5,
            ((5. + transform.translation.abs().z - 2.5) * (transform.translation.signum().z)) as i32 / 5,
        ];
                                             
        if standing_on_block_id != player.block_id {
            let delta_block_id = [
                standing_on_block_id[0] - player.block_id[0],
                standing_on_block_id[1] - player.block_id[1]
            ];
            player.block_id = standing_on_block_id;
            // spawn center of new row/column
            let center_block_id = [
                standing_on_block_id[0] + (delta_block_id[0] * player.view_distance),
                standing_on_block_id[1] + (delta_block_id[1] * player.view_distance)
            ];
            dbg!(center_block_id);
            spawn_block(&mut commands, &mut meshes, &mut materials, center_block_id);
            (1..player.view_distance+1).for_each(|i| {
                let block_id = [
                    center_block_id[0] + delta_block_id[1] * i,
                    center_block_id[1] + delta_block_id[0] * i
                ];
                let block_id2 = [
                    center_block_id[0] + delta_block_id[1] * -i,
                    center_block_id[1] + delta_block_id[0] * -i
                ];
                spawn_block(&mut commands, &mut meshes, &mut materials, block_id);
                spawn_block(&mut commands, &mut meshes, &mut materials, block_id2);
            })
        }
    }
}

fn setup_chunkmap(mut commands: Commands, mut chunk_map: ResMut<ChunkMapResource>) {


    // Insert chunks into the chunk map
    let coordinates1 = Vec2::new(0.0, 0.0);
    // get biome from nearest neighbor in ChunkMapResource.biome_list
    let nearest_biome = chunk_map.biome_list.iter().min_by_key(|(coordinates, _)| {
        (coordinates.x - coordinates1.x).abs() + (coordinates.y - coordinates1.y).abs()
    }).unwrap().1;
    let chunk1 = Chunk::new(coordinates1, nearest_biome);
    chunk_map.chunk_map.insert(coordinates1, chunk1);

    let chunk2 = Chunk { /* Initialize chunk fields */ };
    let coordinates2 = Vec2::new(1.0, 0.0);
    chunk_map.chunk_map.insert(coordinates2, chunk2);

    // Use the chunk map in your systems or plugins
    commands.spawn(chunk_map.clone());
    commands.spawn(chunk_map.clone());
}
