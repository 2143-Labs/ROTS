use std::time::Duration;

use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    server::{Cast, ChangeMovement},
    spells::UpdateSharedComponent,
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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum YourCastResult {
    /// Go ahead with cast
    Ok(NetEntId),
    /// Go ahread with cast, but you had some extra cd to account for
    OffsetBy(Duration, NetEntId),
    /// You can't cast.
    No(Duration),
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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct UnitDie {
    pub id: NetEntId,
    pub disappear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SpawnInteractable {
    pub id: NetEntId,
    pub location: Vec3,
    // TODO
    //pub interaction_type: T
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct DespawnInteractable {
    pub id: NetEntId,
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
