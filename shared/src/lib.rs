use std::{collections::HashMap, env::current_dir, fs::OpenOptions};

use bevy::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub mod animations;
pub mod casting;
pub mod event;
pub mod interactable;
pub mod netlib;
pub mod stats;
pub mod unit;

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
    Fire2,
    Mod1,
    Special1,

    Chat,
}

impl GameAction {
    /// Run condition that returns true if this keycode was just pressed
    pub const fn just_pressed(&'static self) -> impl Fn(Res<ButtonInput<KeyCode>>, Res<Config>) -> bool {
        move |keyboard_input, config| config.just_pressed(&keyboard_input, self.clone())
    }
}

/// Just a tag we have in the shared library for any controlled character
#[derive(Component)]
pub struct AnyUnit;

#[derive(Component)]
pub struct Controlled;

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

    pub fn just_pressed(&self, keyboard_input: &Res<ButtonInput<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.just_pressed(x), ga)
    }

    pub fn pressed(&self, keyboard_input: &Res<ButtonInput<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.pressed(x), ga)
    }

    pub fn just_released(&self, keyboard_input: &Res<ButtonInput<KeyCode>>, ga: GameAction) -> bool {
        self.pressing_keybind(|x| keyboard_input.just_released(x), ga)
    }
}

static DEFAULT_BINDS: Lazy<Keybinds> = Lazy::new(|| {
    HashMap::from([
        (GameAction::MoveForward, vec![KeyCode::KeyW]),
        (GameAction::MoveBackward, vec![KeyCode::KeyS]),
        (GameAction::StrafeLeft, vec![KeyCode::KeyA]),
        (GameAction::StrafeRight, vec![KeyCode::KeyD]),
        (GameAction::RotateLeft, vec![KeyCode::KeyQ]),
        (GameAction::RotateRight, vec![KeyCode::KeyE]),
        (GameAction::Jump, vec![KeyCode::Space]),
        (GameAction::Use, vec![KeyCode::KeyF]),
        (GameAction::ChangeCamera, vec![KeyCode::KeyC]),
        (GameAction::UnlockCursor, vec![KeyCode::KeyX]),
        (GameAction::Fire1, vec![KeyCode::KeyT]),
        (GameAction::Fire2, vec![KeyCode::KeyE]),
        (GameAction::Mod1, vec![KeyCode::ShiftLeft]),
        (GameAction::Chat, vec![KeyCode::Enter]),
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
    pub fn default_config_str() -> String {
        serde_yaml::to_string(&Self::default()).unwrap()
    }

    pub fn debug_keybinds(&self) {
        info!(?self.keybindings);
    }

    pub fn load_from_main_dir() -> Self {
        let mut path = current_dir().unwrap();
        path.push("config.yaml");

        info!("Loading config from {path:?}");
        // Try to open config file
        match OpenOptions::new().read(true).open(&path) {
            Ok(file) => match serde_yaml::from_reader(file) {
                Ok(user_config) => {
                    let mut user_config: Config = user_config;

                    // For each keybind, assign the default if not bound.
                    let mut all_binds = DEFAULT_BINDS.clone();
                    all_binds.extend(user_config.keybindings);
                    user_config.keybindings = all_binds;

                    user_config
                }
                Err(e) => {
                    eprintln!("====================================");
                    eprintln!("===  Failed to load your config  ===");
                    eprintln!("====================================");
                    eprintln!("{:?}", e);
                    eprintln!("Here is the default config:");
                    println!("{}", Self::default_config_str());
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
