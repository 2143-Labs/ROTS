use std::mem::discriminant;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use shared::animations::AnimationTimer;
use shared::animations::CastNetId;
use shared::animations::CastPointTimer;
use shared::event::ERFE;

use shared::event::client::YourCastResult;
use shared::event::spells::ShootingData;
use shared::event::NetEntId;
use shared::netlib::EventToClient;
use shared::netlib::EventToServer;
use shared::{
    event::server::Cast,
    netlib::{send_event_to_server, MainServerEndpoint, ServerResources},
    Config,
};

use crate::cameras::chat::ChatState;
use crate::cameras::notifications::Notification;
use crate::cameras::ClientAimDirection;
use crate::player::Player;
use crate::states::GameState;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartLocalAnimation>()
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
            .add_systems(
                Update,
                (
                    start_local_skill_cast_animation,
                    shared::animations::systems::tick_casts,
                    maybe_cancel_local_skill_animation,
                )
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

pub type GameTime = f64;

#[derive(Event, Debug)]
struct StartLocalAnimation(Cast);

fn cast_skill_click(
    //keyboard_input: Res<Input<KeyCode>>,
    //config: Res<Config>,
    //aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartLocalAnimation>,
) {
    //let (_ent, _ply_face, _transform, _actions) = player.single();
    ev_sa.send(StartLocalAnimation(Cast::Melee));
}

#[derive(Component)]
pub struct CurrentTargetingCursor(pub Option<NetEntId>);

fn cast_skill_2(
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    player: Query<(&Transform, &CurrentTargetingCursor), With<Player>>,
    aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartLocalAnimation>,
) {
    let (transform, target_ent) = player.single();
    let aim_dir = aim_dir.single().0;

    if config.pressed(&keyboard_input, shared::GameAction::Mod1) {
        if let Some(ent) = target_ent.0 {
            let event = Cast::ShootTargeted(transform.translation, ent);
            ev_sa.send(StartLocalAnimation(event));
        } else {
            warn!("Not targeting anything, not sure what to shoot");
        }
    } else if config.pressed(&keyboard_input, shared::GameAction::MoveBackward) {
        let target = transform.translation
            + Vec3 {
                x: aim_dir.cos(),
                y: 0.0,
                z: -aim_dir.sin(),
            } * 30.0;

        let event = Cast::Teleport(target);
        ev_sa.send(StartLocalAnimation(event));
    } else {
        let event = Cast::Aoe(transform.translation);
        ev_sa.send(StartLocalAnimation(event));
    }
}

fn cast_skill_1(
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    player: Query<&Transform, With<Player>>,
    aim_dir: Query<&ClientAimDirection>,
    mut ev_sa: EventWriter<StartLocalAnimation>,
) {
    if config.pressed(&keyboard_input, shared::GameAction::Mod1) {
        let event = Cast::Buff;
        ev_sa.send(StartLocalAnimation(event));
    } else {
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
        ev_sa.send(StartLocalAnimation(event));
    }
}

fn start_local_skill_cast_animation(
    mut ev_sa: EventReader<StartLocalAnimation>,
    player: Query<(Entity, Option<(&AnimationTimer, &Cast)>), With<Player>>,
    mut commands: Commands,
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for StartLocalAnimation(cast) in ev_sa.read() {
        let (player_ent, existing_cast) = player.single();
        trace!(?cast, ?existing_cast, "Attempting to cast");
        let skill_data = cast.get_skill_info();

        let mut can_cast = true;

        // TODO check cooldown
        if let Some((anim_timer, existing_cast_data)) = existing_cast {
            let current_anim_state =
                existing_cast_data.get_current_animation(anim_timer.0.elapsed());

            trace!(?current_anim_state, ?existing_cast_data);

            // TODO should some skills be self-cancelling?
            // cancel the skill if we are already casting it
            if discriminant(existing_cast_data) == discriminant(cast) {
                can_cast = false;
            }
            match current_anim_state {
                shared::animations::AnimationState::FrontSwing => {}
                shared::animations::AnimationState::WindUp => can_cast = false,
                shared::animations::AnimationState::WindDown => can_cast = false,
                shared::animations::AnimationState::Backswing => {}
                shared::animations::AnimationState::Done => {}
            }
        }

        if !can_cast {
            debug!(?cast, "Could not cast because of existing cast animation");
            continue;
        }

        let event = EventToServer::Cast(cast.clone());
        send_event_to_server(&sr.handler, mse.0, &event);

        commands
            .entity(player_ent)
            .remove::<(AnimationTimer, Cast)>()
            .insert((
                cast.clone(),
                AnimationTimer(Timer::new(skill_data.get_total_duration(), TimerMode::Once)),
                CastPointTimer(Timer::new(skill_data.get_cast_point(), TimerMode::Once)),
            ));
    }
}

fn maybe_cancel_local_skill_animation(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    mut skill_cast_results: ERFE<YourCastResult>,
    mut notifs: EventWriter<Notification>,
) {
    for skill_cast_result in skill_cast_results.read() {
        trace!(?skill_cast_result);
        match skill_cast_result.event {
            YourCastResult::Ok(new_cast_id) => {
                // server says we can keep casting, insert the new id we got
                commands
                    .entity(player.single())
                    .insert(CastNetId(new_cast_id));
            }
            YourCastResult::OffsetBy(_, _) => todo!(),
            YourCastResult::No(tl) => {
                notifs.send(Notification(format!("Skill is on cooldown! {tl:?}")));

                // we got denied, stop casting, refund everything
                commands
                    .entity(player.single())
                    .remove::<(AnimationTimer, Cast, CastPointTimer)>();
            }
        }
    }
}
