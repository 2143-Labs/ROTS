use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    server::{Cast, ChangeMovement},
    NetEntId, PlayerData,
};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    pub your_name: String,
    pub your_id: NetEntId,
    pub players: Vec<PlayerConnected>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
    pub initial_transform: Transform,
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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SomeoneCast {
    pub caster_id: NetEntId,
    pub cast_id: NetEntId,
    pub cast: Cast,
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
