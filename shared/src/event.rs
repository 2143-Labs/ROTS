use bevy::prelude::*;

use bevy_ecs::system::EntityCommands;
use message_io::network::Endpoint;
use serde::{Deserialize, Serialize};

use crate::stats::Health;

use self::spells::NPC;

pub mod client;
pub mod server;
pub mod spells;

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetEntId(pub u64);
impl NetEntId {
    pub fn random() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug, Clone, Event)]
pub struct EventFromEndpoint<E> {
    pub event: E,
    pub endpoint: Endpoint,
}

/// Event Reader with endpoint data.
pub type ERFE<'w, 's, E> = EventReader<'w, 's, EventFromEndpoint<E>>;

impl<E> EventFromEndpoint<E> {
    pub fn new(endpoint: Endpoint, e: E) -> Self {
        EventFromEndpoint { event: e, endpoint }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnitType {
    Player { name: String },
    NPC { npc_type: NPC },
}
    //fn components(&self, e: &mut EntityCommands) {
        //match self {
            //UnitType::Player { name } => {
                //e.insert(
            //},
            //UnitType::NPC { npc_type } => todo!(),
        //}
    //}
//}

// This is all the data need to initialize a player for the client side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitData {
    pub unit: UnitType,
    pub ent_id: NetEntId,
    pub health: Health,
    pub transform: Transform,
}
