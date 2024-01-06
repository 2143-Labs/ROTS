use bevy::prelude::*;
use shared::{
    event::{
        client::{DespawnInteractable, SpawnInteractable},
        NetEntId, ERFE,
    },
    interactable::Interactable,
};

use crate::states::GameState;

pub struct InteractablePlugin;
impl Plugin for InteractablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_interact_spawn, on_interact_despawn).run_if(in_state(GameState::ClientConnected)),
        );
    }
}

fn on_interact_spawn(
    mut pd: ERFE<SpawnInteractable>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in pd.read() {
        let event = &event.event;
        let cube = PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.7, 0.7, 0.0).into()),
            transform: Transform::from_translation(event.location),
            ..Default::default()
        };

        commands.spawn((
            cube,
            event.id,
            //Name::new(format!("Interactable: {:?}", npc_type)),
            Interactable,
        ));
    }
}

fn on_interact_despawn(
    mut pd: ERFE<DespawnInteractable>,
    units: Query<(Entity, &NetEntId), With<Interactable>>,
    mut commands: Commands,
) {
    for event in pd.read() {
        let event = &event.event;
        for (unit_ent, net_ent) in &units {
            if &event.id == net_ent {
                commands.entity(unit_ent).despawn_recursive();
            }
        }
    }
}
