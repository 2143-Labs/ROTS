use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Component, Debug, Clone, Copy)]
pub struct MovementIntention(pub Vec2);

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Eq)]
pub enum AttackIntention {
    None,
    // TODO: For now, all NPC attacks are held here, tightly coupled but easier to test with
    AutoAttack(Timer),
}
