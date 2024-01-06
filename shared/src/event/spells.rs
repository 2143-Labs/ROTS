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
    Mage
}

#[derive(Component)]
pub enum AIType {
    None,
    WalkToNearestPlayer,
}

impl NPC {
    pub fn model(&self) -> &'static str {
        match self {
            NPC::Penguin => "penguinwalk.gltf#Scene0",
            NPC::Mage => "bookmageIdle.gltf#Scene0",
        }
    }

    pub fn get_base_health(&self) -> Health {
        Health(match self {
            NPC::Penguin => 50,
        })
    }

    pub fn get_ai_component(&self) -> AIType {
        match self {
            NPC::Penguin => AIType::WalkToNearestPlayer,
            NPC::Mage => AIType::WalkToNearestPlayer
        }
    }
}
