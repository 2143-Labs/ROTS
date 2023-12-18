use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::netlib::ServerResources;
use crate::event::EventFromEndpoint;

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    // TODO
    pub your_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
    // TODO
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
