use std::time::Duration;

use bevy::prelude::*;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameManagerState>()
            .insert_resource(SpawnTimer(Timer::new(
                Duration::from_secs(1),
                TimerMode::Repeating,
            )))
            .add_systems(
                Update,
                (tick_game_manager).run_if(in_state(GameManagerState::Playing)),
            );
    }
}

use rand::Rng;
use shared::event::{client::SpawnUnit, spells::NPC, NetEntId, UnitData};

#[derive(Resource, Clone, Debug)]
pub struct SpawnTimer(Timer);

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameManagerState {
    #[default]
    Warmup,
    Playing,
    NotPlaying,
}

fn tick_game_manager(
    mut spawn_npc: EventWriter<SpawnUnit>,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time<Virtual>>,
) {
    timer.0.tick(time.delta());

    for _ in 0..timer.0.times_finished_this_tick() {
        let enemy_type = NPC::Penguin;
        let location = Vec3::new(
            rand::thread_rng().gen_range(-25.0..25.0),
            0.,
            rand::thread_rng().gen_range(-25.0..25.0),
        );
        spawn_npc.send(SpawnUnit {
            data: UnitData {
                unit: shared::event::UnitType::NPC {
                    npc_type: enemy_type.clone(),
                },
                ent_id: NetEntId(rand::random()),
                health: enemy_type.get_base_health(),
                transform: Transform::from_translation(location),
            },
        });
    }
}
