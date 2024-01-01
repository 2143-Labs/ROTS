use super::event::NetEntId;
use bevy::prelude::*;

pub enum AnimationState {
    /// Still cancelable
    FrontSwing,
    /// not cancelable. skill will be cast at the end of this animation
    WindUp,
    // at the end of the windup the skill gets cast
    /// Forced part of the animation backswing
    WindDown,
    /// Optional part of the animation backswing
    Backswing,
}


