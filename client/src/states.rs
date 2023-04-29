use bevy::{prelude::*, window::CursorGrabMode};
use bevy_asset_loader::prelude::*;
use bevy_fly_camera::FlyCamera;

use crate::{
    player::PlayerSpriteAssets,
    setup::{CameraFollow, Hideable, MuscleManAssets}, networking::client_bullet_receiver::{NetPlayerSprite, ProjectileSheet},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_state::<FreeCamState>()
            .add_state::<PhysView>()
            .add_loading_state(
                LoadingState::new(GameState::Loading).continue_to_state(GameState::Ready),
            )
            .add_collection_to_loading_state::<_, PlayerSpriteAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_, NetPlayerSprite>(GameState::Loading)
            .add_collection_to_loading_state::<_, ProjectileSheet>(GameState::Loading)
            .add_collection_to_loading_state::<_, MuscleManAssets>(GameState::Loading)
            .add_system(toggle_freecam)
            .add_system(toggle_phyics_debug_view);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum FreeCamState {
    #[default]
    Locked,
    Free,
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

pub fn toggle_freecam(
    mut players: Query<Entity, With<CameraFollow>>,
    mut commands: Commands,
    cam_state: Res<State<FreeCamState>>,
    mut next_state: ResMut<NextState<FreeCamState>>,
    input: Res<Input<KeyCode>>,
    mut windows_query: Query<&mut Window>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = windows_query.get_single_mut() {
            window.cursor.grab_mode = match window.cursor.grab_mode {
                CursorGrabMode::None => CursorGrabMode::Locked,
                CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
            };
            window.cursor.visible = !window.cursor.visible;
        };
        println!("::: ESCAPE PRESSED! :::");
        next_state.set(match cam_state.0 {
            FreeCamState::Free => {
                println!("::: FreeCamState::Free :::");
                for player in players.iter_mut() {
                    commands.entity(player).remove::<FlyCamera>();
                }
                FreeCamState::Locked
            }
            FreeCamState::Locked => {
                println!("::: FreeCamState::Locked :::");
                for player in players.iter_mut() {
                    commands.entity(player).insert(FlyCamera::default());
                }
                FreeCamState::Free
            }
        });
    }
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
