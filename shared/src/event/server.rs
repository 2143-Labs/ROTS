use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{spells::ShootingData, NetEntId};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ConnectRequest {
    pub name: Option<String>,
    pub my_location: Transform,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct Heartbeat {}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum Cast {
    Teleport(Vec3),
    Shoot(ShootingData),
    ShootTargeted(NetEntId),
    Aoe(Vec3),
    Melee,
    Buff,
}

/// walking and stuff
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum ChangeMovement {
    StandStill,
    Move2d(Vec2),
    SetTransform(Transform),
}

include!(concat!(env!("OUT_DIR"), "/server_event.rs"));
