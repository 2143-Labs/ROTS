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

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum, Component, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum NPC {
    Penguin,
}

#[derive(Component)]
pub enum AIType {
    None,
    WalkToNearestPlayer,
    //TurnToNearestPlayer,
}

impl NPC {
    pub fn model(&self) -> &'static str {
        match self {
            NPC::Penguin => "penguin.gltf#Scene0",
        }
    }

    pub fn get_ai_component(&self) -> AIType {
        match self {
            NPC::Penguin => AIType::WalkToNearestPlayer,

        }
    }
}
