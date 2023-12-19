use std::time::Duration;

use crate::{
    cameras::{notifications::Notification, thirdperson::PLAYER_SPEED},
    player::{Player, PlayerName, MovementIntention},
    states::GameState,
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use shared::{
    event::{
        client::{PlayerConnected, PlayerDisconnected, SomeoneMoved, WorldData},
        server::{ChangeMovement, ConnectRequest, Heartbeat},
        NetEntId, ERFE,
    },
    netlib::{
        send_event_to_server, setup_client, EventToClient, EventToServer, MainServerEndpoint,
        ServerResources,
    },
    Config,
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        shared::event::client::register_events(app);
        app.add_event::<SpawnOtherPlayer>()
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
                    spawn_player,
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
    mse: Res<MainServerEndpoint>,
    config: Res<Config>,
) {
    let event = EventToServer::ConnectRequest(ConnectRequest {
        name: config.name.clone(),
    });
    send_event_to_server(&sr.handler, mse.0, &event);
    info!("Sent connection packet to {}", mse.0);
}

fn receive_world_data(
    mut world_data: ERFE<WorldData>,
    mut commands: Commands,
    mut notif: EventWriter<Notification>,
    local_player: Query<Entity, With<Player>>,
    mut spawn_player: EventWriter<SpawnOtherPlayer>,
) {
    for event in world_data.read() {
        info!(?event, "Server has returned world data!");

        let my_name = &event.event.your_name;
        let my_id = event.event.your_id;

        // Add our netentid + name
        commands
            .entity(local_player.single())
            .insert(my_id)
            .insert(PlayerName(my_name.clone()));

        notif.send(Notification(format!(
            "Connected to server as {my_name} {my_id:?}"
        )));

        for other_player_data in &event.event.players {
            notif.send(Notification(format!("Connected: {other_player_data:?}")));
            spawn_player.send(SpawnOtherPlayer(PlayerConnected {
                data: other_player_data.clone(),
            }));
            info!(?other_player_data);
        }
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
    mut other_players: Query<(&NetEntId, &mut Transform, &mut MovementIntention), With<OtherPlayer>>,
) {
    for movement in someone_moved.read() {
        info!(?movement);
        for (ply_net, mut ply_tfm, mut ply_intent) in &mut other_players {
            info!(?ply_net);
            if &movement.event.id == ply_net {
                match movement.event.movement {
                    ChangeMovement::SetTransform(t) => *ply_tfm = t,
                    ChangeMovement::StandStill => {
                    },
                    ChangeMovement::Move2d(intent) => {
                        *ply_intent = MovementIntention(intent);
                    },
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
        ply_tfm.translation += Vec3::new(ply_intent.0.x, 0.0, ply_intent.0.y) * PLAYER_SPEED * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct OtherPlayer;

#[derive(Event)]
pub struct SpawnOtherPlayer(PlayerConnected);

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut er: EventReader<SpawnOtherPlayer>,
) {
    for SpawnOtherPlayer(event) in er.read() {
        let cube = PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.4, 0.7, 0.1).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        };

        commands.spawn((
            cube,
            OtherPlayer,
            PlayerName(event.data.name.clone()),
            MovementIntention(Vec2::ZERO),
            Name::new(format!("Player: {}", event.data.name)),
            // their NetEntId is a component
            event.data.ent_id,
        ));
    }
}

fn on_connect(
    mut c_info: ERFE<PlayerConnected>,
    mut notif: EventWriter<Notification>,

    mut spawn_player: EventWriter<SpawnOtherPlayer>,
) {
    for event in c_info.read() {
        notif.send(Notification(format!("{:?}", event.event)));
        spawn_player.send(SpawnOtherPlayer(event.event.clone()));
        info!(?event);
    }
}
