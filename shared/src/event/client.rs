use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{NetEntId, PlayerData, server::ChangeMovement};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    pub your_name: String,
    pub your_id: NetEntId,
    pub players: Vec<PlayerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
    pub data: PlayerData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerDisconnected {
    pub id: NetEntId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SomeoneMoved {
    pub id: NetEntId,
    pub movement: ChangeMovement,
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
