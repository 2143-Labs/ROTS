use crate::event::EventFromEndpoint;
use crate::netlib::ServerResources;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::spells::{UpdateSharedComponent, NPC};
use super::{
    server::{Cast, ChangeMovement},
    NetEntId, PlayerData,
};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct WorldData {
    pub your_player_data: PlayerData,
    pub players: Vec<PlayerConnected>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerConnected {
    pub data: PlayerData,
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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SpawnNPC {
    pub location: Vec3,
    pub npc: NPC,
}

include!(concat!(env!("OUT_DIR"), "/client_event.rs"));
