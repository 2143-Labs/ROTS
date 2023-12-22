use bevy::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShootingData {
    pub shot_from: Vec3,
    pub target: Vec3,
}
