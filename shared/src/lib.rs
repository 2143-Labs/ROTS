use std::sync::{Arc, Mutex};

use bevy::prelude::Resource;
use message_io::network::Endpoint;
use serde::{Serialize, Deserialize};
pub mod event {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerConnect {
        pub name: String
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GameNetEvent {
    Noop,
    PlayerConnect(event::PlayerConnect),
    PlayerList(Vec<event::PlayerConnect>),
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ServerResources {
    pub event_list: Arc<Mutex<Vec<(Endpoint, GameNetEvent)>>>,
}
