use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::netlib::ServerResources;

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    // TODO
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
    // TODO
}

fn a() {
    //ServerResources
    let s = EventWriter;
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));

