use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ConnectRequest {
    pub name: Option<String>,
}

include!(concat!(env!("OUT_DIR"), "/server_event.rs"));
