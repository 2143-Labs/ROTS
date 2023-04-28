use serde::{Serialize, Deserialize};
pub mod event {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerConnect;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameNetEvent {
    Noop,
    PlayerConnect(event::PlayerConnect),
}

