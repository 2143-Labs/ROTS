use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::NetEntId;

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    pub your_name: String,
    pub your_id: NetEntId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerDisconnected {
    pub id: NetEntId
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
