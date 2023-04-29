use std::{sync::{Arc, Mutex}, fs::OpenOptions, env::current_dir};

use bevy::prelude::*;
use event::AnimationThing;
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdatePos {
        pub id: NetEntId,
        pub transform: Transform,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ShootBullet {
        pub id: NetEntId,
        pub phys: BulletPhysics,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Animation {
        pub id: NetEntId,
        pub animation: AnimationThing,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AnimationThing {
        Waterball,
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
    YouAre(event::PlayerInfo),
    PlayerConnect(event::PlayerInfo),
    PlayerList(Vec<event::PlayerInfo>),
    UpdatePos(event::UpdatePos),
    ShootBullet(event::ShootBullet),
    Animation(event::Animation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToServer {
    Noop,
    Connect{name: String},
    UpdatePos(Transform),
    ShootBullet(BulletPhysics),
    BeginAnimation(AnimationThing),
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

#[derive(Resource, Deserialize, Serialize)]
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

        // Try to open config file
        match OpenOptions::new().read(true).open(&path) {
            Ok(file) => {
                serde_yaml::from_reader(file).unwrap()
            },
            Err(kind) => match kind.kind() {
                //if it doesn't exist, try to create it.
                std::io::ErrorKind::NotFound => {
                    let config = Self {
                        ip: "john2143.com".into(),
                        port: 25565,
                        name: None,
                    };

                    let file_handler = OpenOptions::new().create(true).write(true).open(&path).unwrap();

                    serde_yaml::to_writer(file_handler, &config).unwrap();
                    // should mabye just crash here and ask them to review their config
                    config
                },
                e => panic!("Failed to open config file {e:?}"),
            },
        }
    }
}
