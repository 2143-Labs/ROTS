use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::netlib::ServerResources;
use crate::event::EventFromEndpoint;

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ConnectRequest {
    pub name: Option<String>,
}

include!(concat!(env!("OUT_DIR"), "/server_event.rs"));
