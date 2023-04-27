use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use crate::{states::GameState, player::Player, setup::Hideable};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_system(gen_world.run_if(in_state(GameState::Ready)))
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
