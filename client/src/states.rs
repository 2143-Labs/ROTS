use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    networking::client_bullet_receiver::{NetPlayerSprite, ProjectileSheet},
    player::PlayerSpriteAssets,
    setup::{Hideable, MuscleManAssets},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_state::<CameraState>()
            .add_state::<PhysView>()
            .add_loading_state(
                LoadingState::new(GameState::Loading).continue_to_state(GameState::Ready),
            )
            .add_collection_to_loading_state::<_, PlayerSpriteAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_, NetPlayerSprite>(GameState::Loading)
            .add_collection_to_loading_state::<_, ProjectileSheet>(GameState::Loading)
            .add_collection_to_loading_state::<_, MuscleManAssets>(GameState::Loading)
            .add_system(toggle_phyics_debug_view);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum CameraState {
    #[default]
    ThirdPersonLocked,
    ThirdPersonFreeMouse,
    Free,
    TopDown,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum PhysView {
    #[default]
    Normal,
    Debug,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    NetworkConnecting,
    Ready,
}

pub fn toggle_phyics_debug_view(
    mut vis_query: Query<&mut Visibility, With<Hideable>>,
    phys_state: Res<State<PhysView>>,
    mut next_state: ResMut<NextState<PhysView>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F1) {
        next_state.set(match phys_state.0 {
            PhysView::Normal => {
                println!("::: PhysView::Debug :::");
                for mut pbr in &mut vis_query.iter_mut() {
                    *pbr = Visibility::Hidden;
                }
                PhysView::Debug
            }
            PhysView::Debug => {
                println!("::: PhysView::Normal :::");
                for mut pbr in &mut vis_query.iter_mut() {
                    *pbr = Visibility::Visible;
                }
                PhysView::Normal
            }
        });
    }
}
