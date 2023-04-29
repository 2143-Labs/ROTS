use std::ops::DerefMut;

use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};
use message_io::network::{NetEvent, Transport, Endpoint};
use rand::{thread_rng, Rng};
use shared::{event::{PlayerInfo, UpdatePos, ShootBullet}, ServerResources, EventFromEndpoint, EventToClient, EventToServer, NetEntId, Config};

use crate::{lifetime::{Lifetime}, states::GameState, sprites::AnimationTimer, player::{FaceCamera, PlayerSpriteAssets}};

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
            .add_systems((
                    send_movement_updates,
                    tick_net_client,
                    get_movement_updates,
                    on_player_connect,
                    on_player_shoot).distributive_run_if(in_state(GameState::Ready)));
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
    mut players: Query<(&mut Transform, &NetEntId), With<NetworkPlayer>>,
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

#[derive(AssetCollection, Resource)]
pub struct NetPlayerSprite {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 44.))]
    #[asset(texture_atlas(columns = 2, rows = 1))]
    #[asset(path = "MrMan.png")]
    pub run: Handle<TextureAtlas>,
}


#[derive(Component)]
pub struct NetworkPlayer {
    name: String,
}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerInfo>>,
    mut commands: Commands,
    sprite_res: Res<NetPlayerSprite>,
    mut sprite_params: Sprite3dParams,
) {
    for e in &mut ev_player_connect {
        info!("TODO spawn player in world... {e:?}");

        let sprite = AtlasSprite3d {
            atlas: sprite_res.run.clone(),

            pixels_per_metre: 44.,
            partial_alpha: true,
            unlit: true,

            index: 1,

            transform: Default::default(),
            // pivot: Some(Vec2::new(0.5, 0.5)),
            ..default()
        }
        .bundle(&mut sprite_params);

        commands
            .spawn(sprite)
            .insert(e.event.id)
            .insert(NetworkPlayer {
                name: e.event.name.clone(),
            })
            .insert(AnimationTimer(Timer::from_seconds(
                0.4,
                TimerMode::Repeating,
            )));
    }
}

#[derive(AssetCollection, Resource)]
pub struct ProjectileSheet{
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    #[asset(texture_atlas(columns = 1, rows = 1))]
    #[asset(path = "Banana.png")]
    pub banana: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128.))]
    #[asset(texture_atlas(columns = 25, rows = 1))]
    #[asset(path = "orb-Sheet.png")]
    pub fireball: Handle<TextureAtlas>,
}

fn on_player_shoot(
    mut ev_player_shoot: EventReader<EventFromEndpoint<ShootBullet>>,
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    proj_res: Res<ProjectileSheet>,
    mut sprite_params: Sprite3dParams,
) {
    for e in &mut ev_player_shoot {
        info!("spawning bullet");

        let sprite = AtlasSprite3d {
            atlas: proj_res.banana.clone(),
            pixels_per_metre: 64.,
            partial_alpha: true,
            unlit: true,
            index: 0,
            ..default()
        }
        .bundle(&mut sprite_params);

        commands
            .spawn(sprite)
            .insert(Lifetime {
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            })
            .insert(e.event.phys.clone())
            .insert(e.event.id)
            .insert(FaceCamera)
            .insert(AnimationTimer(Timer::from_seconds(
                        0.1,
                        TimerMode::Repeating,
            )));

    }
}
