use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy::math::Quat;

#[derive(Serialize, Deserialize, Component, Debug, Clone, Copy)]
pub struct MovementIntention(pub Vec2);

#[derive(Serialize, Deserialize, Component, Debug, Clone, Copy)]
pub struct TurningIntention(pub Quat);