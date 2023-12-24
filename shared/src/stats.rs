use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Component, Debug, Eq, PartialEq, Clone, Copy)]
pub struct Health(pub u32);

impl Default for Health {
    fn default() -> Self {
        Self(3)
    }
}
