use std::{sync::{Arc, Mutex}, fs::OpenOptions, env::current_dir};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToClient {
    Noop,
    PlayerConnect(event::PlayerInfo),
    PlayerList(Vec<event::PlayerInfo>),
    UpdatePos(NetEntId, Transform),
    ShootBullet(NetEntId, BulletPhysics),

}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToServer {
    Noop,
    Connect{name: String},
    UpdatePos(Transform),
    ShootBullet(BulletPhysics),
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

#[derive(Resource, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub name: Option<String>,
}

impl Config {
    pub fn load_from_main_dir() -> Self {
        let mut path = current_dir().unwrap();
        path.pop();
        path.push("config.yaml");

        let file = OpenOptions::new().read(true).open(path).unwrap();
        serde_yaml::from_reader(file).unwrap()
    }
}
