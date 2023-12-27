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

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct SpawnNPC {
    pub location: Vec3,
    pub npc: NPC,
}

