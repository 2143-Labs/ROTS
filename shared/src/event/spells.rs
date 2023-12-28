use crate::stats::Health;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ShootingData {
    pub shot_from: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateSharedComponent {
    Health(Health),
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum, Component)]
pub enum NPC {
    Penguin,
}

impl NPC {
    pub fn model(&self) -> &'static str {
        match self {
            NPC::Penguin => "penguin.gltf#Scene0",
        }
    }
}
