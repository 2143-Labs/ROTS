use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Component, Debug, Clone, Copy)]
pub struct MovementIntention(pub Vec2);
