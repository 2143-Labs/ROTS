use std::ops::DerefMut;

use bevy::prelude::*;
use message_io::network::{NetEvent, Transport, Endpoint};
use rand::{thread_rng, Rng};
use shared::{event::{PlayerInfo, UpdatePos, ShootBullet}, ServerResources, EventFromEndpoint, EventToClient, EventToServer, NetEntId, Config};

use crate::lifetime::{Lifetime};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::load_from_main_dir();
        let (server, endpoint) = setup_networking_server(&config);
        app
            .insert_resource(config)
            .insert_resource(server)
            .insert_resource(endpoint)
            .add_event::<EventFromEndpoint<PlayerInfo>>()
            .add_event::<EventFromEndpoint<UpdatePos>>()
            .add_event::<EventFromEndpoint<ShootBullet>>()
            .add_system(on_player_connect)
            .add_system(on_player_shoot)
            .add_system(tick_net_client)
            .add_system(send_movement_updates)
            .add_system(get_movement_updates);
    }
}

#[derive(Resource)]
pub(crate) struct MainServerEndpoint(pub Endpoint);

fn setup_networking_server(config: &Config) -> (ServerResources<EventToClient>, MainServerEndpoint) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let con_str = (&*config.ip, config.port);

    let (server, _) = handler.network().connect(Transport::Udp, con_str).expect("Failed to connect ot server");

    info!("probably connected");

    let name = match &config.name {
        Some(name) => name.clone(),
        None => {
            let random_id = thread_rng().gen_range(1..10000);
            format!("Player #{random_id}")
        }
    };

    let connect_event = EventToServer::Connect { name };
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
    mut movement_events: EventReader<EventFromEndpoint<UpdatePos>>,
    mut players: Query<(&mut Transform, &NetEntId)>,
){
    let events: Vec<_> = movement_events.iter().collect();
    //info!(?events);
    for (mut player_transform, &net_id) in &mut players {
        for event in &events {
            if event.event.id == net_id {
                *player_transform = event.event.transform;
            }
        }
    }
}

pub fn tick_net_client(
    event_list_res: Res<ServerResources<EventToClient>>,
    mut ev_player_connect: EventWriter<EventFromEndpoint<PlayerInfo>>,
    mut ev_player_movement: EventWriter<EventFromEndpoint<UpdatePos>>,
    mut ev_player_shoot: EventWriter<EventFromEndpoint<ShootBullet>>,
) {
    let events_to_process: Vec<_> = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for (_endpoint, e) in events_to_process {
        match e {
            EventToClient::Noop => warn!("Got noop event"),
            EventToClient::PlayerConnect(p) => ev_player_connect.send(EventFromEndpoint::new(_endpoint, p)),
            EventToClient::PlayerList(p_list) => ev_player_connect.send_batch(p_list.into_iter().map(|x| EventFromEndpoint::new(_endpoint, x))),
            EventToClient::UpdatePos(e) => ev_player_movement.send(EventFromEndpoint::new(_endpoint, e)),
            EventToClient::ShootBullet(e) => ev_player_shoot.send(EventFromEndpoint::new(_endpoint, e)),
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
            .insert(e.event.id);
    }
}

fn on_player_shoot(
    mut ev_player_shoot: EventReader<EventFromEndpoint<ShootBullet>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for e in &mut ev_player_shoot {
        info!("spawning bullet");

        let color = match e.event.phys.ai {
            shared::BulletAI::Direct => Color::BLACK,
            shared::BulletAI::Wavy => Color::BLUE,
            shared::BulletAI::Wavy2 => Color::RED,
        };

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(0.2))),
                material: materials.add(color.into()),
                transform: Default::default(),
                ..default()
            })
            .insert(Lifetime {
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            })
            .insert(e.event.phys.clone())
            .insert(e.event.id)
            .insert(Name::new("Bullet"));
    }
}
