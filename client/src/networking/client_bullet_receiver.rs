use std::ops::DerefMut;

use bevy::prelude::*;
use message_io::network::{NetEvent, Transport, Endpoint};
use rand::{thread_rng, Rng};
use shared::{event::PlayerInfo, ServerResources, EventFromEndpoint, EventToClient, EventToServer, NetEntId};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let (server, endpoint) = setup_networking_server();
        app
            .insert_resource(server)
            .insert_resource(endpoint)
            .add_event::<EventFromEndpoint<PlayerInfo>>()
            .add_event::<EventFromEndpoint<(NetEntId, Transform)>>()
            .add_system(on_player_connect)
            .add_system(tick_net_client)
            .add_system(send_movement_updates)
            .add_system(get_movement_updates);
    }
}

#[derive(Resource)]
struct MainServerEndpoint(Endpoint);

fn setup_networking_server() -> (ServerResources<EventToClient>, MainServerEndpoint) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let (server, _) = handler.network().connect(Transport::Udp, "127.0.0.1:3042").expect("Failed to connect ot server");

    info!("probably connected");

    let name = thread_rng().gen_range(1..10000);

    let connect_event = EventToServer::Connect { name: format!("Player #{name}") };
    let event_json = serde_json::to_string(&connect_event).unwrap();
    handler.network().send(server, event_json.as_bytes());

    info!("sent json");

    let res = ServerResources {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let res_copy = res.clone();

    let mse = MainServerEndpoint(server.clone());

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {},
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //let s = from_utf8(data);
                //info!(?s);
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            }
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

    (res, mse)
}

fn send_movement_updates(
    player_query: Query<&Transform, With<crate::player::Player>>,
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
){
    if let Ok(transform) = player_query.get_single() {
        let ev = EventToServer::UpdatePos(*transform);
        let data = serde_json::to_string(&ev).unwrap();
        event_list_res.handler.network().send(mse.0, data.as_bytes());
    }
}

fn get_movement_updates(
    mut movement_events: EventReader<EventFromEndpoint<(NetEntId, Transform)>>,
    mut players: Query<(&mut Transform, &NetEntId)>,
){
    let events: Vec<_> = movement_events.iter().collect();
    //info!(?events);
    for (mut player_transform, &net_id) in &mut players {
        for event in &events {
            if event.event.0 == net_id {
                *player_transform = event.event.1;
            }
        }
    }
}

pub fn tick_net_client(
    event_list_res: Res<ServerResources<EventToClient>>,
    mut ev_player_connect: EventWriter<EventFromEndpoint<PlayerInfo>>,
    mut ev_player_movement: EventWriter<EventFromEndpoint<(NetEntId, Transform)>>,
) {
    let events_to_process: Vec<_> = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for (_endpoint, e) in events_to_process {
        match e {
            EventToClient::Noop => warn!("Got noop event"),
            EventToClient::PlayerConnect(p) => ev_player_connect.send(EventFromEndpoint::new(_endpoint, p)),
            EventToClient::PlayerList(p_list) => ev_player_connect.send_batch(p_list.into_iter().map(|x| EventFromEndpoint::new(_endpoint, x))),
            EventToClient::UpdatePos(player_id, transform) => ev_player_movement.send(EventFromEndpoint::new(_endpoint, (player_id, transform))),
            _ => {},
        }
    }
}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerInfo>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for e in &mut ev_player_connect {
        info!("TODO spawn player in world... {e:?}");

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
                material: materials.add(Color::OLIVE.into()),
                transform: Default::default(),
                ..default()
            })
            .insert(e.event.id)
        ;

        //commands
            //.spawn(sprite)
            //.insert(RigidBody::Dynamic)
            //.insert(Collider::cuboid(0.5, 0.5, 0.2))
            //.insert(LockedAxes::ROTATION_LOCKED)
            //.insert(GravityScale(1.))
            //.insert(ColliderMassProperties::Mass(1.0))
            //.insert(Name::new("PlayerSprite"))
            //.insert(Player::default())
            //.insert(FaceCamera)
            //.insert(Jumper {
                //cooldown: 0.5,
                //timer: Timer::from_seconds(1., TimerMode::Once),
            //})
            //.insert(Name::new("PlayerBody"))
            //.insert(AnimationTimer(Timer::from_seconds(
                //0.4,
                //TimerMode::Repeating,
            //)));
        //}
    }
}