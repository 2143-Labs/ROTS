use bevy::prelude::*;

use message_io::network::Endpoint;
use serde::{Deserialize, Serialize};


pub mod client;
pub mod server;


#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetEntId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulletAI {
    /// Bullet directly travels from point to point
    Direct,
    Wavy,
    Wavy2,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct BulletPhysics {
    pub fired_from: Vec2,
    pub fired_target: Vec2,
    // Tiles per second
    pub speed: f32,
    pub ai: BulletAI,
    //fired_time: time_since_start,
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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerInfo {
    pub name: String,
    pub id: NetEntId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct UpdatePos {
    pub id: NetEntId,
    pub transform: Transform,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ShootBullet {
    pub id: NetEntId,
    pub phys: BulletPhysics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Component, Event)]
pub struct Animation {
    pub id: NetEntId,
    pub animation: AnimationThing,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct PlayerDisconnect {
    pub id: NetEntId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum AnimationThing {
    Waterball,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct Heartbeat {
    pub id: NetEntId,
}
