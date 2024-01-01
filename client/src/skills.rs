use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use shared::animations::AnimationTimer;
use shared::event::spells::ShootingData;
use shared::netlib::EventToClient;
use shared::netlib::EventToServer;
use shared::{
    event::server::Cast,
    netlib::{send_event_to_server, MainServerEndpoint, ServerResources},
    Config,
};

use crate::cameras::chat::ChatState;
use crate::cameras::ClientAimDirection;
use crate::player::Player;
use crate::states::GameState;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartAnimation>()
            .add_systems(
                Update,
                cast_skill_1
                    .run_if(shared::GameAction::Fire1.just_pressed())
                    .run_if(in_state(ChatState::NotChatting))
                    .run_if(any_with_component::<Player>()),
            )
            .add_systems(
                Update,
                cast_skill_2
                    .run_if(shared::GameAction::Fire2.just_pressed())
                    .run_if(in_state(ChatState::NotChatting))
                    .run_if(any_with_component::<Player>()),
            )
            .add_systems(
                Update,
                cast_skill_click
                    .run_if(input_just_pressed(MouseButton::Left))
                    .run_if(in_state(ChatState::NotChatting))
                    .run_if(any_with_component::<Player>()),
            )
            .add_systems(Update, (start_local_skill_cast_animation, tick_anim_timers))
            .add_systems(
                Update,
                (send_network_packet).run_if(in_state(GameState::ClientConnected)),
            );
    }
}

pub type GameTime = f64;

#[derive(Event, Debug)]
struct StartAnimation(Cast);

fn cast_skill_click(
    //keyboard_input: Res<Input<KeyCode>>,
    //config: Res<Config>,
    //aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartAnimation>,
) {
    //let (_ent, _ply_face, _transform, _actions) = player.single();
    ev_sa.send(StartAnimation(Cast::Melee));
}

fn cast_skill_2(
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    player: Query<&Transform, With<Player>>,
    aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartAnimation>,
) {
    let transform = player.single();
    let aim_dir = aim_dir.single().0;

    if config.pressed(&keyboard_input, shared::GameAction::MoveBackward) {
        let target = transform.translation
            + Vec3 {
                x: aim_dir.cos(),
                y: 0.0,
                z: -aim_dir.sin(),
            } * 10.0;

        let event = Cast::Teleport(target);
        ev_sa.send(StartAnimation(event));
    }
}

fn cast_skill_1(
    //keyboard_input: Res<Input<KeyCode>>,
    //config: Res<Config>,
    player: Query<&Transform, With<Player>>,
    aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartAnimation>,
) {
    let transform = player.single();

    let aim_dir = aim_dir.single().0;

    let target = transform.translation
        + Vec3 {
            x: aim_dir.cos(),
            y: 0.0,
            z: -aim_dir.sin(),
        };

    let shooting_data = ShootingData {
        shot_from: transform.translation,
        target,
    };
    let event = Cast::Shoot(shooting_data);
    ev_sa.send(StartAnimation(event));
}

fn tick_anim_timers(
    mut all_timers: Query<(Entity, &mut AnimationTimer)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    for (ent, mut timer) in &mut all_timers {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            trace!("Finished backswing");
            commands.entity(ent).remove::<Cast>().remove::<AnimationTimer>();
        }
    }
}

fn start_local_skill_cast_animation(
    mut ev_sa: EventReader<StartAnimation>,
    player: Query<(Entity, Option<(&AnimationTimer, &Cast)>), With<Player>>,
    mut commands: Commands,
) {
    for StartAnimation(cast) in ev_sa.read() {
        let (player_ent, existing_cast) = player.single();
        trace!(?cast, ?existing_cast, "Attempting to cast");
        let skill_data = cast.get_skill_info();

        let mut can_cast = true;

        // TODO check cooldown
        if let Some((anim_timer, existing_cast_data)) = existing_cast {

            let current_anim_state = existing_cast_data.get_current_animation(anim_timer.0.elapsed());
            info!(?current_anim_state);
            match current_anim_state {
                shared::animations::AnimationState::FrontSwing => {},
                shared::animations::AnimationState::WindUp => can_cast = false,
                shared::animations::AnimationState::WindDown => can_cast = false,
                shared::animations::AnimationState::Backswing => {},
                shared::animations::AnimationState::Done => {},
            }
        }

        if !can_cast {
            warn!(?cast, "Could not cast because of existing cast animation");
            continue;
        }

        commands
            .entity(player_ent)
            .remove::<(AnimationTimer, Cast)>()
            .insert((
                cast.clone(),
                AnimationTimer(Timer::new(skill_data.get_total_duration(), TimerMode::Once)),
            ));
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
