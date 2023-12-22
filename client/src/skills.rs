use bevy::prelude::*;
use shared::event::server::ChangeMovement;
use shared::event::spells::ShootingData;
use shared::netlib::EventToClient;
use shared::netlib::EventToServer;
use shared::{
    event::server::Cast,
    netlib::{send_event_to_server, MainServerEndpoint, ServerResources},
    Config,
};

use crate::cameras::ClientAimDirection;
use crate::states::GameState;
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
            (start_local_skill_cast_animation,),
        )
        .add_systems(
            Update,
            (send_network_packet)
                    .run_if(in_state(GameState::ClientConnected)),
        )
        .add_event::<StartAnimation>();
    }
}

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
struct StartAnimation(Cast);

/// Run condition that returns true if this keycode was just pressed
const fn just_pressed(ga: shared::GameAction) -> impl Fn(Res<Input<KeyCode>>, Res<Config>) -> bool {
    move |keyboard_input, config| config.just_pressed(&keyboard_input, ga.clone())
}

fn cast_skills(
    player: Query<(Entity, &Player, &Transform, Option<&Actions>)>,
    aim_dir: Query<&ClientAimDirection>,
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

    let aim_dir = aim_dir.single().0;
    let target = _transform.translation + Vec3 { x: aim_dir.sin(), y: 0.0, z: aim_dir.cos() };

    let shooting_data = ShootingData {
        shot_from: _transform.translation,
        target,
    };
    let event = Cast::Shoot(shooting_data);
    info!(?event);
    ev_sa.send(StartAnimation(event));
}

fn start_local_skill_cast_animation(
    mut ev_sa: EventReader<StartAnimation>,
    //our_transform: Query<Entity, With<Player>>,
    //mut commands: Commands,
) {
    for _ev in ev_sa.read() {
        //commands.entity(our_transform.single()).insert(bundle);
    }
}

fn send_network_packet(
    mut ev_sa: EventReader<StartAnimation>,
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for ev in ev_sa.read() {
        let event = EventToServer::Cast(ev.0.clone());
        send_event_to_server(&sr.handler, mse.0, &event);
    }
}
