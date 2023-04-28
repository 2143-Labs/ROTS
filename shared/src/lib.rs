use std::{sync::{Arc, Mutex}, ops::DerefMut};

use bevy::prelude::*;
use message_io::{network::Endpoint, node::NodeHandler};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetEntId(pub u64);

pub mod event {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerInfo {
        pub name: String,
        pub id: NetEntId,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToClient {
    Noop,
    PlayerConnect(event::PlayerInfo),
    PlayerList(Vec<event::PlayerInfo>),
    UpdatePos(NetEntId, Transform),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToServer {
    Noop,
    Connect{name: String},
    UpdatePos(Transform),
}

#[derive(Debug, Clone)]
pub struct EventFromEndpoint<E> {
    pub event: E,
    pub endpoint: Endpoint,
}

impl<E> EventFromEndpoint<E> {
    pub fn new(endpoint: Endpoint, e: E) -> Self {
        EventFromEndpoint { event: e, endpoint }
    }
}

#[derive(Resource, Clone)]
pub struct ServerResources<T> {
    pub event_list: Arc<Mutex<Vec<(Endpoint, T)>>>,
    pub handler: NodeHandler<()>,
}
