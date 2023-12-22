use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::NetEntId;

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ShootingData {
    pub shot_from: Vec3,
    pub target: Vec3,
}
