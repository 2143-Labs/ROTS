use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_rapier3d::prelude::Collider;
use rand::Rng;

use crate::{states::GameState, player::Player, setup::Hideable};

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    /// Creates a new vector.
    #[inline(always)]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub fn init(app: &mut App) -> &mut App {
    app
        .insert_resource(BlockMapResource::default())
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
struct Block {
    position: Vec2i,
    biome: Arc<Biome>,
    height_variation: f32,
    visible: bool,
    // Other block data...
}

impl Block {
    fn new(position: Vec2i, biome: Arc<Biome>) -> Self {
        Block {
            position,
            height_variation: biome.default_height_variation(),
            biome,
            visible: false,
            // Initialize other block data...
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
struct BlockMapResource {
    block_map: HashMap<Vec2i, Block>,
    biome_list: Vec<(Vec2i, Biome)>,
}

impl Default for BlockMapResource {
    fn default() -> Self {
        BlockMapResource {
            block_map: HashMap::default(),
            biome_list: vec![(
                Vec2i::new(0, 0),
                Biome::Forest,
            ),
            (
                Vec2i::new(10, 30),
                Biome::Desert,
            ),
            (
                Vec2i::new(-20, -5),
                Biome::Mountains,
            ),
            (
                Vec2i::new(-6,30),
                Biome::Ocean,
            ),
            (
                Vec2i::new(21, -40),
                Biome::Plains,
            )],
        }   
    }
}

pub fn spawn_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block_id: &Vec2i
) {
    let size = 5.;
    let odd_block = (block_id.x + block_id.y).abs() %2i32 == 1i32;
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
            block_id.x as f32 * size,
            -1.,
            block_id.y as f32 * size
        )))
        .insert(Hideable)
        .insert(Name::new(format!("Block_{}_{}", block_id.x, block_id.y)));
}

pub fn gen_world(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut block_map_res: ResMut<BlockMapResource>,
) {
    for (transform, mut player) in player_query.iter_mut() {
        let standing_on_block_id: [i32; 2] = [
            (transform.translation.x / 10.0).floor() as i32,
            (transform.translation.z / 10.0).floor() as i32,
        ];
        // if inserting new block
        if standing_on_block_id != player.block_id {
            player.block_id = standing_on_block_id;
            // iterate over every block within radius of players viewdistance (cirlce)
            for x in (standing_on_block_id[0] - player.view_distance)..=(standing_on_block_id[0] + player.view_distance)
            {
                for y in (standing_on_block_id[1] - player.view_distance)..=(standing_on_block_id[1] + player.view_distance)
                {
                    // if block is outside of viewdistance
                    if (x - standing_on_block_id[0]).pow(2) + (y - standing_on_block_id[1]).pow(2) > player.view_distance.pow(2) {
                        continue;
                    }
                    let blockid = Vec2i::new(x,y);
                    // if block does not exist
                    if !block_map_res.block_map.contains_key(&blockid) {
                        // spawn block
                        spawn_block(&mut commands, &mut meshes, &mut materials, &blockid);
                        // get biome from nearest neighbor in BlockMapResource.biome_list
                        let nearest_biome = Arc::new(block_map_res.biome_list.iter().min_by_key(|(coordinates, _)| {
                            (coordinates.x - blockid.x).abs() + (coordinates.y - blockid.y).abs()
                        }).unwrap().1);
                        let block = Block::new(blockid.clone(), nearest_biome);
                        block_map_res.block_map.insert(blockid, block);
                    }
                }
            }
        }
    }

}

fn init_blockmap(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut block_map_res: ResMut<BlockMapResource>) {
    // Insert blocks into the block map
    let blockid= Vec2i::new(0, 0);
    // get biome from nearest neighbor in BlockMapResource.biome_list
    let nearest_biome = Arc::new(block_map_res.biome_list.iter().min_by_key(|(coordinates, _)| {
        (coordinates.x - blockid.x).abs() + (coordinates.y - blockid.y).abs()
    }).unwrap().1);
    let block1 = Block::new(blockid.clone(), nearest_biome);
    spawn_block(&mut commands, &mut meshes, &mut materials, &blockid);
    block_map_res.block_map.insert(blockid, block1);
}
