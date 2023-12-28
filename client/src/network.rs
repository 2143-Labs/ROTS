use std::time::Duration;

use crate::{
    cameras::{notifications::Notification, thirdperson::PLAYER_SPEED},
    cli::CliArgs,
    network::stats::HPIndicator,
    player::{MovementIntention, Player, PlayerName},
    states::GameState,
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use shared::{
    event::{
        client::{PlayerDisconnected, SomeoneMoved, SpawnUnit, WorldData},
        server::{ChangeMovement, ConnectRequest, Heartbeat},
        NetEntId, ERFE,
    },
    netlib::{
        send_event_to_server, setup_client, EventToClient, EventToServer, MainServerEndpoint,
        ServerResources,
    },
    AnyUnit, Config,
};

mod casting;
pub mod npc;
pub mod stats;

#[derive(Component)]
pub struct OtherPlayer;

pub struct NetworkingPlugin;
impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        shared::event::client::register_events(app);
        app.add_plugins(casting::CastingNetworkPlugin)
            .add_plugins((stats::StatsNetworkPlugin, npc::NPCPlugin))
            .add_event::<SpawnUnit>()
            .add_systems(
                OnEnter(GameState::ClientConnecting),
                (
                    // Setup the client and immediatly advance the state
                    setup_client::<EventToClient>,
                    |mut state: ResMut<NextState<GameState>>| state.set(GameState::ClientConnected),
                ),
            )
            .add_systems(OnEnter(GameState::ClientConnected), (send_connect_packet,))
            .add_systems(
                Update,
                (
                    shared::event::client::drain_events,
                    receive_world_data,
                    on_connect,
                    on_disconnect,
                    on_someone_move,
                    go_movement_intents,
                )
                    .run_if(in_state(GameState::ClientConnected)),
            )
            .add_systems(
                Update,
                (send_movement, send_interp)
                    .run_if(on_timer(Duration::from_millis(25)))
                    .run_if(in_state(GameState::ClientConnected)),
            )
            .add_systems(
                Update,
                send_heartbeat
                    .run_if(on_timer(Duration::from_millis(200)))
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn send_connect_packet(
    sr: Res<ServerResources<EventToClient>>,
    args: Res<CliArgs>,
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
    local_player: Query<&Transform, With<Player>>,
) {
    let my_location = *local_player.single();
    let event = EventToServer::ConnectRequest(ConnectRequest {
        name: args.name_override.clone().or(config.name.clone()),
        my_location,
    });
    send_event_to_server(&sr.handler, mse.0, &event);
    info!("Sent connection packet to {}", mse.0);
}

fn build_healthbar(
    s: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let player_id = s.parent_entity();
    // spawn their hp bar
    let mut hp_bar = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.9, 0.3, 0.0).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.4, 0.0)),
        ..Default::default()
    };

    // make it invisible until it's updated
    hp_bar.transform.scale = Vec3::ZERO;

    s.spawn((hp_bar, crate::network::stats::HPBar(player_id)));
}

fn receive_world_data(
    mut world_data: ERFE<WorldData>,
    mut commands: Commands,
    mut notif: EventWriter<Notification>,
    mut local_player: Query<(Entity, &mut Transform), With<Player>>,
    mut spawn_units: EventWriter<SpawnUnit>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    for event in world_data.read() {
        info!(?event, "Server has returned world data!");

        let my_id = event.event.your_unit_id;
        for unit in &event.event.unit_data {
            match &unit.unit {
                shared::event::UnitType::Player { name } => {
                    // If it's a player, check to see if it is us
                    if unit.ent_id == my_id {
                        // If so, start aligning the client to it
                        let (p_ent, mut p_tfm) = local_player.single_mut();
                        p_tfm.translation = unit.transform.translation;

                        notif.send(Notification(format!(
                            "Connected to server as {name} {my_id:?}"
                        )));

                        // Add our netentid + name
                        commands
                            .entity(p_ent)
                            .insert(my_id)
                            .insert(PlayerName(name.clone()))
                            .insert(unit.health)
                            .with_children(|s| build_healthbar(s, &mut meshes, &mut materials));

                        // if this is us, skip the spawn units call cause we updated a local unit
                        // instead. TODO eventually fix this so when we fully despawn the menu
                        // player unit
                        continue;
                    } else {
                        // Not the local player
                        notif.send(Notification(format!("Connected: {name}")));
                    }
                }
                _ => {}
            }

            //For any unit that isnt us, spawn it
            spawn_units.send(SpawnUnit {
                data: unit.clone(),
            });
        }

        commands.spawn((
            HPIndicator::HP,
            TextBundle::from_section(
                "HP: #",
                TextStyle {
                    font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                    font_size: 45.0,
                    color: Color::rgb(0.4, 0.5, 0.75),
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            }),
        ));
        commands.spawn((
            HPIndicator::Deaths,
            TextBundle::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                    font_size: 45.0,
                    color: Color::rgb(0.9, 0.2, 0.2),
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(50.0),
                ..default()
            }),
        ));
    }
}

fn send_heartbeat(sr: Res<ServerResources<EventToClient>>, mse: Res<MainServerEndpoint>) {
    let event = EventToServer::Heartbeat(Heartbeat {});
    send_event_to_server(&sr.handler, mse.0, &event);
}

fn send_interp(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    our_transform: Query<&MovementIntention, (With<Player>, Changed<MovementIntention>)>,
) {
    if let Ok(intent) = our_transform.get_single() {
        let event = EventToServer::ChangeMovement(ChangeMovement::Move2d(intent.0));
        send_event_to_server(&sr.handler, mse.0, &event);
    }
}

fn send_movement(
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
    our_transform: Query<&Transform, (With<Player>, Changed<Transform>)>,
) {
    if let Ok(transform) = our_transform.get_single() {
        let event = EventToServer::ChangeMovement(ChangeMovement::SetTransform(transform.clone()));
        send_event_to_server(&sr.handler, mse.0, &event);
    }
}

fn on_disconnect(
    mut dc_info: ERFE<PlayerDisconnected>,
    mut notif: EventWriter<Notification>,
    mut commands: Commands,
    // TODO what if the server is disconnecting us?
    other_players: Query<(Entity, &NetEntId, &PlayerName), With<OtherPlayer>>,
) {
    for event in dc_info.read() {
        let disconnected_ent_id = event.event.id;
        for (player_ent, net_ent_id, PlayerName(player_name)) in &other_players {
            if net_ent_id == &disconnected_ent_id {
                notif.send(Notification(format!("{player_name} Disconnected.")));
                commands.entity(player_ent).despawn_recursive();
            }
        }
        info!(?disconnected_ent_id);
    }
}

fn on_someone_move(
    mut someone_moved: ERFE<SomeoneMoved>,
    mut other_players: Query<
        (&NetEntId, &mut Transform, &mut MovementIntention),
        With<OtherPlayer>,
    >,
) {
    for movement in someone_moved.read() {
        for (ply_net, mut ply_tfm, mut ply_intent) in &mut other_players {
            if &movement.event.id == ply_net {
                match movement.event.movement {
                    ChangeMovement::SetTransform(t) => *ply_tfm = t,
                    ChangeMovement::StandStill => {}
                    ChangeMovement::Move2d(intent) => {
                        *ply_intent = MovementIntention(intent);
                    }
                }
            }
        }
    }
}

fn go_movement_intents(
    mut other_players: Query<(&mut Transform, &MovementIntention), With<OtherPlayer>>,
    time: Res<Time>,
) {
    for (mut ply_tfm, ply_intent) in &mut other_players {
        ply_tfm.translation +=
            Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * PLAYER_SPEED * time.delta_seconds();
    }
}

fn on_connect(
    mut c_info: ERFE<SpawnUnit>,
    mut notif: EventWriter<Notification>,

    mut local_spawn_unit: EventWriter<SpawnUnit>,
) {
    for event in c_info.read() {
        notif.send(Notification(format!("{:?}", event.event)));
        local_spawn_unit.send(event.event.clone());
    }
}
