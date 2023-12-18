use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
