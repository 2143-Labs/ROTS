use crate::{event::EventFromEndpoint, unit::AttackIntention};
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
pub struct SendChat {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct Heartbeat {}

#[derive(Debug, Clone, Serialize, Deserialize, Event, Component)]
pub enum Cast {
    Teleport(Vec3),
    Shoot(ShootingData),
    ShootTargeted(Vec3, NetEntId),
    Melee,
    Aoe(Vec3),
    Buff,
}

/// walking and stuff
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum ChangeMovement {
    StandStill,
    Move2d(Vec2),
    SetTransform(Transform),
    AttackIntent(AttackIntention),
}

include!(concat!(env!("OUT_DIR"), "/server_event.rs"));
