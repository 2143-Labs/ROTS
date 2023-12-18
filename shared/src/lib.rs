use std::{
    collections::HashMap,
    env::current_dir,
    fs::OpenOptions,
    sync::{Arc, Mutex},
};

pub mod events;

use bevy::prelude::*;
use event::AnimationThing;
use message_io::{network::Endpoint, node::NodeHandler};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone)]
pub struct EventList<T: Clone> {
    pub event_list: Arc<Mutex<Vec<(Endpoint, T)>>>,
}

#[derive(Resource, Clone)]
pub struct ServerNodeHandler {
    pub handler: NodeHandler<()>,
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetEntId(pub u64);

pub mod event {
    use super::*;

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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[non_exhaustive]
pub enum EventToClient {
    Noop,
    YouAre(event::PlayerInfo),
    PlayerConnect(event::PlayerInfo),
    PlayerList(Vec<event::PlayerInfo>),
    UpdatePos(event::UpdatePos),
    ShootBullet(event::ShootBullet),
    Animation(event::Animation),
    PlayerDisconnect(event::PlayerDisconnect),
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[non_exhaustive]
pub enum EventToServer {
    Noop,
    Connect { name: Option<String> },
    UpdatePos(Transform),
    ShootBullet(BulletPhysics),
    BeginAnimation(AnimationThing),
    Heartbeat,
}

#[derive(Debug, Clone, Event)]
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

//fn default_qe_sens() -> f32 {
//3.0
//}

//fn default_sens() -> f32 {
//0.003
//}

#[derive(Reflect, Hash, Eq, PartialEq, Clone, Deserialize, Serialize, Debug)]
pub enum GameAction {
    MoveForward,
    MoveBackward,
    StrafeRight,
    StrafeLeft,
    RotateRight,
    RotateLeft,
    Use,
    Jump,
    ChangeCamera,
    UnlockCursor,

    Fire1,
    Special1,
}

#[derive(Reflect, Clone, Resource, Deserialize, Serialize, Debug)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub name: Option<String>,
    //#[serde(default="default_sens")]
    pub sens: f32,
    //#[serde(default="default_qe_sens")]
    pub qe_sens: f32,

    pub keybindings: Keybinds, // TODO rust_phf
}

type Keybinds = HashMap<GameAction, Vec<KeyCode>>;

impl Config {
    pub fn pressing_keybind(
        &self,
        mut keyboard_input: impl FnMut(KeyCode) -> bool,
        ga: GameAction,
    ) -> bool {
        let bound_key_codes = match self.keybindings.get(&ga) {
            Some(b) => b,
            None => DEFAULT_BINDS.get(&ga).unwrap(),
        };

        for key in bound_key_codes {
            if keyboard_input(*key) {
                return true;
            }
        }

        false
    }

    pub fn just_pressed(&self, keyboard_input: &Res<Input<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.just_pressed(x), ga)
    }

    pub fn pressed(&self, keyboard_input: &Res<Input<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.pressed(x), ga)
    }

    pub fn just_released(&self, keyboard_input: &Res<Input<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.just_released(x), ga)
    }
}

static DEFAULT_BINDS: Lazy<Keybinds> = Lazy::new(|| {
    HashMap::from([
        (GameAction::MoveForward, vec![KeyCode::W]),
        (GameAction::MoveBackward, vec![KeyCode::S]),
        (GameAction::StrafeLeft, vec![KeyCode::A]),
        (GameAction::StrafeRight, vec![KeyCode::D]),
        (GameAction::RotateLeft, vec![KeyCode::Q]),
        (GameAction::RotateRight, vec![KeyCode::E]),
        (GameAction::Jump, vec![KeyCode::Space]),
        (GameAction::Use, vec![KeyCode::F]),
        (GameAction::ChangeCamera, vec![KeyCode::C]),
        (GameAction::UnlockCursor, vec![KeyCode::X]),
        (GameAction::Fire1, vec![KeyCode::T]),
    ])
});

impl Default for Config {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: 25565,
            sens: 0.003,
            qe_sens: 3.0,
            name: None,
            keybindings: DEFAULT_BINDS.clone(),
        }
    }
}

pub struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::load_from_main_dir();
        app.insert_resource(config).register_type::<Config>();
    }
}

impl Config {
    pub fn load_from_main_dir() -> Self {
        let mut path = current_dir().unwrap();
        path.push("config.yaml");

        info!("Loading config from {path:?}");
        // Try to open config file
        match OpenOptions::new().read(true).open(&path) {
            Ok(file) => match serde_yaml::from_reader(file) {
                Ok(v) => v,
                Err(e) => {
                    error!("====================================");
                    error!("===  Failed to load your config  ===");
                    error!("====================================");
                    error!(?e);
                    error!("Here is the default config:");
                    let default_config = serde_yaml::to_string(&Self::default()).unwrap();
                    println!("{}", default_config);
                    panic!("Please fix the above error and restart your program");
                }
            },
            Err(kind) => match kind.kind() {
                //if it doesn't exist, try to create it.
                std::io::ErrorKind::NotFound => {
                    let config = Self::default();

                    let file_handler = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&path)
                        .unwrap();

                    serde_yaml::to_writer(file_handler, &config).unwrap();
                    // should mabye just crash here and ask them to review their config
                    config
                }
                e => panic!("Failed to open config file {e:?}"),
            },
        }
    }
}
