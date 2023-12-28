use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    spells::UpdateSharedComponent,
    server::{Cast, ChangeMovement},
    NetEntId, UnitData,
};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    pub your_unit_id: NetEntId,
    pub unit_data: Vec<UnitData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SpawnUnit {
    pub data: UnitData,
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

#[derive(Debug, Clone, Serialize, Deserialize, Event, Hash, PartialEq, Eq)]
pub struct BulletHit {
    pub bullet: NetEntId,
    pub player: NetEntId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SomeoneUpdateComponent {
    pub id: NetEntId,
    pub update: UpdateSharedComponent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct Chat {
    pub source: Option<NetEntId>,
    pub text: String,
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
