use std::time::Duration;

use crate::event::{client::SomeoneCast, server::Cast, NetEntId};

use bevy::prelude::*;

pub mod systems;

#[derive(Debug)]
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
    Done,
}

#[derive(Debug)]
pub struct SkillInfo {
    pub frontswing: Duration,
    pub windup: Duration,
    pub winddown: Duration,
    pub backswing: Duration,

    pub cooldown: Duration,
}

#[derive(Component, Debug)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct CastPointTimer(pub Timer);

#[derive(Component)]
pub struct CastNetId(pub NetEntId);

#[derive(Event)]
pub struct DoCast(pub SomeoneCast);

macro_rules! skill_info {
    (cd $cd:expr => [ fs $fs:expr ; wu $wu:expr ; wd $wd:expr ; bs $bs: expr ]) => {
        SkillInfo {
            frontswing: Duration::from_secs_f32($fs),
            windup: Duration::from_secs_f32($wu),
            winddown: Duration::from_secs_f32($wd),
            backswing: Duration::from_secs_f32($bs),

            cooldown: Duration::from_secs_f32($cd),
        }
    };
}

impl Cast {
    pub fn get_skill_info(&self) -> SkillInfo {
        match self {
            Cast::Teleport(_) => skill_info!(cd 5.0 => [ fs 1.0 ; wu 1.0 ; wd 1.0 ; bs 1.0 ]),
            Cast::Shoot(_) => skill_info!(cd 0.5 => [ fs 0.1 ; wu 0.0 ; wd 0.1 ; bs 0.0 ]),
            Cast::ShootTargeted(_) => skill_info!(cd 1.0 => [ fs 0.5 ; wu 0.0 ; wd 0.1 ; bs 0.0 ]),
            Cast::Melee => skill_info!(cd 1.0 => [ fs 0.2 ; wu 0.0 ; wd 0.1 ; bs 0.3 ]),
            Cast::Aoe(_) => skill_info!(cd 5.0 => [ fs 1.0 ; wu 1.0 ; wd 1.0 ; bs 1.0 ]),
            Cast::Buff => skill_info!(cd 30.0 => [ fs 0.25 ; wu 0.75 ; wd 0.0 ; bs 0.0 ]),
        }
    }

    pub fn get_current_animation(&self, mut time: Duration) -> AnimationState {
        let skill = self.get_skill_info();

        if time < skill.frontswing {
            return AnimationState::FrontSwing;
        }
        time -= skill.frontswing;

        if time < skill.windup {
            return AnimationState::WindUp;
        }
        time -= skill.windup;

        if time < skill.winddown {
            return AnimationState::WindDown;
        }
        time -= skill.winddown;

        if time < skill.backswing {
            return AnimationState::Backswing;
        }

        AnimationState::Done
    }
}

impl SkillInfo {
    /// Duration until the skill is complete
    pub fn get_total_duration(&self) -> Duration {
        self.frontswing + self.windup + self.winddown + self.backswing
    }

    /// Duration until the skill is actually cast on the server (eg does damage or whatever)
    pub fn get_cast_point(&self) -> Duration {
        self.frontswing + self.windup
    }

    pub fn get_free_point(&self) -> Duration {
        self.frontswing + self.windup + self.winddown
    }
}
