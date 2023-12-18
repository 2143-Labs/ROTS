use bevy::prelude::*;
use shared::Config;

use crate::{cameras::notifications::Notification, player::Player};

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cast_skills.run_if(just_pressed(shared::GameAction::Fire1)),
        )
        .add_systems(
            Update,
            (start_local_skill_cast_animation, send_network_packet),
        )
        .add_event::<StartAnimation>();
    }
}

#[derive(Debug, Clone)]
struct SkillData;

pub type GameTime = f64;

#[derive(Component)]
pub enum Actions {
    AnimationWindup,
    AnimationBackswing,
}

impl Actions {
    fn is_cancellable(&self) -> bool {
        return true;
    }
}

#[derive(Event, Debug)]
struct StartAnimation(SkillData);

/// Run condition that returns true if this keycode was just pressed
const fn just_pressed(ga: shared::GameAction) -> impl Fn(Res<Input<KeyCode>>, Res<Config>) -> bool {
    move |keyboard_input, config| config.just_pressed(&keyboard_input, ga.clone())
}

fn cast_skills(
    player: Query<(Entity, &Player, &Transform, Option<&Actions>)>,
    mut ev_sa: EventWriter<StartAnimation>,
) {
    let (_ent, _ply_face, _transform, actions) = player.single();

    match actions {
        Some(a @ Actions::AnimationWindup) => {
            if !a.is_cancellable() {
                // Can't go
                return;
            }
        }
        Some(Actions::AnimationBackswing) => {
            //Ok
            //cancel backswing?
        }
        None => {
            //Ok
        }
    }

    ev_sa.send(StartAnimation(SkillData));
}

fn start_local_skill_cast_animation(mut ev_sa: EventReader<StartAnimation>) {
    for _ev in ev_sa.read() {}
}

fn send_network_packet(
    mut ev_sa: EventReader<StartAnimation>,
    mut ev_notif: EventWriter<Notification>,
) {
    for ev in ev_sa.read() {
        ev_notif.send(Notification(format!(
            "This is a test notification {:?}",
            ev
        )));
        // TODO send netowrk packet to say that we are casting a skill
    }
}
