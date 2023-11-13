use std::{ops::DerefMut, time::Duration};

use bevy::{
    prelude::*,
};
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams};
use message_io::network::{Endpoint, NetEvent, Transport};
use rand::{thread_rng, Rng};
use shared::{
    event::{Animation, PlayerDisconnect, PlayerInfo, ShootBullet, UpdatePos},
    Config, EventFromEndpoint, EventToClient, EventToServer, NetEntId, ServerResources,
};

use crate::{
    lifetime::Lifetime,
    player::{FaceCamera, Player},
    sprites::AnimationTimer,
    states::GameState,
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::load_from_main_dir();
        app.add_state::<NetworkingState>()
            .insert_resource(config)
            .register_type::<Config>()
            .add_event::<EventFromEndpoint<PlayerInfo>>()
            .add_event::<EventFromEndpoint<UpdatePos>>()
            .add_event::<EventFromEndpoint<ShootBullet>>()
            .add_event::<EventFromEndpoint<Animation>>()
            .add_event::<EventFromEndpoint<PlayerDisconnect>>()
            .add_systems(OnEnter(GameState::Ready), on_game_ready)
            .add_systems(
                OnEnter(NetworkingState::BeginSetup),
                setup_networking_server,
            )
            .add_systems(
                Update,
                (tick_net_client, on_player_disconnect)
                    .distributive_run_if(in_state(NetworkingState::WaitingForServer)),
            )
            // .add_systems(
            //     FixedUpdate,
            //     send_heartbeat.run_if(on_fixed_timer(Duration::from_millis(250))),
            // )
            .add_systems(
                Update,
                (
                    send_movement_updates,
                    get_movement_updates,
                    on_player_connect,
                    on_player_disconnect,
                    on_player_animate,
                    keep_animation_on_player,
                    tick_net_client,
                    on_player_disconnect,
                    on_player_shoot,
                )
                    .distributive_run_if(in_state(NetworkingState::Connected)),
            );
    }
}

#[derive(Resource)]
pub(crate) struct MainServerEndpoint(pub Endpoint);

#[derive(Clone, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum NetworkingState {
    #[default]
    Disconnected,
    BeginSetup,
    WaitingForServer,
    Connected,
}

fn on_game_ready(mut networking_state: ResMut<NextState<NetworkingState>>) {
    networking_state.set(NetworkingState::BeginSetup);
}

fn setup_networking_server(
    config: Res<Config>,
    mut networking_state: ResMut<NextState<NetworkingState>>,
    mut commands: Commands,
) {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let con_str = (&*config.ip, config.port);

    let (server, _) = handler
        .network()
        .connect(Transport::Udp, con_str)
        .expect("Failed to connect ot server");

    info!("probably connected");

    let res = ServerResources::<EventToClient> {
        handler: handler.clone(),
        event_list: Default::default(),
    };

    let res_copy = res.clone();

    let mse = MainServerEndpoint(server.clone());

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {}
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

    commands.insert_resource(res);
    commands.insert_resource(mse);
    networking_state.set(NetworkingState::WaitingForServer);
}

fn send_movement_updates(
    player_query: Query<&Transform, With<crate::player::Player>>,
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    if let Ok(transform) = player_query.get_single() {
        let ev = EventToServer::UpdatePos(*transform);
        let data = serde_json::to_string(&ev).unwrap();
        event_list_res
            .handler
            .network()
            .send(mse.0, data.as_bytes());
    }
}

fn get_movement_updates(
    mut movement_events: EventReader<EventFromEndpoint<UpdatePos>>,
    mut players: Query<(&mut Transform, &NetEntId), (With<NetworkPlayer>, Without<Player>)>,
) {
    let events: Vec<_> = movement_events.read().collect();
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
    mut ev_player_animation: EventWriter<EventFromEndpoint<Animation>>,
    mut ev_player_disconnect: EventWriter<EventFromEndpoint<PlayerDisconnect>>,

    player: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut networking_state: ResMut<NextState<NetworkingState>>,
) {
    let events_to_process: Vec<_> =
        std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for (_endpoint, e) in events_to_process {
        match e {
            EventToClient::Noop => warn!("Got noop event"),
            EventToClient::PlayerConnect(p) => {
                ev_player_connect.send(EventFromEndpoint::new(_endpoint, p))
            }
            EventToClient::PlayerList(p_list) => ev_player_connect.send_batch(
                p_list
                    .into_iter()
                    .map(|x| EventFromEndpoint::new(_endpoint, x)),
            ),
            EventToClient::UpdatePos(e) => {
                ev_player_movement.send(EventFromEndpoint::new(_endpoint, e))
            }
            EventToClient::ShootBullet(e) => {
                ev_player_shoot.send(EventFromEndpoint::new(_endpoint, e))
            }
            EventToClient::Animation(a) => {
                ev_player_animation.send(EventFromEndpoint::new(_endpoint, a))
            }
            EventToClient::PlayerDisconnect(a) => {
                ev_player_disconnect.send(EventFromEndpoint::new(_endpoint, a))
            }
            EventToClient::YouAre(info) => {
                info!("The server has returned our networking info {info:?}");
                commands
                    .entity(player.single())
                    .insert(NetworkPlayer { name: info.name })
                    .insert(info.id);

                networking_state.set(NetworkingState::Connected);
            }
            _ => {}
        }
    }
}

//#[derive(AssetCollection, Resource)]
//pub struct NetPlayerSprite {
    //#[asset(texture_atlas(tile_size_x = 32., tile_size_y = 44.))]
    //#[asset(texture_atlas(columns = 2, rows = 1))]
    //#[asset(path = "MrMan.png")]
    //pub run: Handle<TextureAtlas>,
//}

#[derive(Component)]
pub struct NetworkPlayer {
    name: String,
}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerInfo>>,
    mut commands: Commands,
    sprite_res: Res<AssetServer>,
    atlases: ResMut<Assets<TextureAtlas>>,
    mut sprite_params: Sprite3dParams,
) {
    for e in &mut ev_player_connect.read() {
        info!("TODO spawn player in world... {e:?}");

        let texture_handle = sprite_res.load("MrMan.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 44.0), 2, 1, None, None);

        let sprite = AtlasSprite3d {
            atlas: atlases.add(texture_atlas),

            pixels_per_metre: 44.,
            alpha_mode: AlphaMode::Add,
            unlit: false,

            index: 1,

            transform: Default::default(),
            // pivot: Some(Vec2::new(0.5, 0.5)),
            ..default()
        }
        .bundle(&mut sprite_params);
        commands.spawn((
            sprite,
            e.event.id,
            NetworkPlayer {
                name: e.event.name.clone(),
            },
            FaceCamera,
            AnimationTimer(Timer::from_seconds(0.4, TimerMode::Repeating)),
        ));
    }
}

fn on_player_disconnect(
    mut ev_player_disconnect: EventReader<EventFromEndpoint<PlayerDisconnect>>,
    mut commands: Commands,
    sprite_res: Query<(Entity, &NetEntId, &NetworkPlayer, Option<&Player>)>,
    mut networking_state: ResMut<NextState<NetworkingState>>,
) {
    for e in &mut ev_player_disconnect.read() {
        for (ent, net_ent, player_info, is_local_player) in &sprite_res {
            if net_ent == &e.event.id {
                warn!("A player has disconnected: {} ({e:?})", player_info.name);
                commands.entity(ent).despawn_recursive();

                if is_local_player.is_some() {
                    warn!("We have been disconnected from the server.");
                    networking_state.set(NetworkingState::Disconnected);
                }
            }
        }
    }
}

//#[derive(AssetCollection, Resource)]
//pub struct ProjectileSheet {
    //#[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32.))]
    //#[asset(texture_atlas(columns = 1, rows = 1))]
    //#[asset(path = "Banana.png")]
    //pub banana: Handle<TextureAtlas>,

    //#[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128.))]
    //#[asset(texture_atlas(columns = 25, rows = 1))]
    //#[asset(path = "orb-Sheet.png")]
    //pub fireball: Handle<TextureAtlas>,

    //#[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128.))]
    //#[asset(texture_atlas(columns = 32, rows = 1))]
    //#[asset(path = "waterboll2-Sheet.png")]
    //pub waterboll: Handle<TextureAtlas>,
//}

fn on_player_shoot(
    mut ev_player_shoot: EventReader<EventFromEndpoint<ShootBullet>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //proj_res: Res<ProjectileSheet>,
    //mut sprite_params: Sprite3dParams,
) {
    for e in &mut ev_player_shoot.read() {
        //info!("spawning bullet");

        //let sprite = AtlasSprite3d {
        //atlas: proj_res.waterboll.clone(),
        //pixels_per_metre: 32.,
        //partial_alpha: true,
        //unlit: false,
        //index: 0,
        //..default()
        //}
        //.bundle(&mut sprite_params);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(0.3))),
                material: materials.add(Color::BLUE.into()),
                transform: Transform::from_xyz(0.0, -100.0, 0.0),
                ..default()
            },
            //.spawn(sprite)
            Lifetime {
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            },
            e.event.phys.clone(),
            e.event.id,
        ));
        //.insert(AnimationTimer(Timer::from_seconds(
        //0.1,
        //TimerMode::Repeating,
        //)));
    }
}

#[derive(Component)]
struct AttachedAnimation(NetEntId);

fn on_player_animate(
    mut ev_player_animate: EventReader<EventFromEndpoint<Animation>>,
    mut commands: Commands,
    //proj_res: Res<ProjectileSheet>,
    mut sprite_params: Sprite3dParams,
) {
    for e in &mut ev_player_animate.read() {
        info!("starting animation {:?}", e.event.animation);

        let frames = 36;

        //let sprite = match e.event.animation {
            //shared::event::AnimationThing::Waterball => AtlasSprite3d {
                //atlas: proj_res.waterboll.clone(),
                //pixels_per_metre: 16.,
                //unlit: false,
                //index: 0,
                //..default()
            //}
            //.bundle(&mut sprite_params),
        //};

        //commands.spawn((
            //sprite,
            //crate::lifetime::LifetimeWithEvent {
                //timer: Timer::from_seconds(0.9, TimerMode::Once),
            //},
            //FaceCamera,
            //AttachedAnimation(e.event.id),
            //AnimationTimer(Timer::from_seconds(
                //1.0 / frames as f32,
                //TimerMode::Repeating,
            //)),
        //));
    }
}

fn keep_animation_on_player(
    players: Query<(&Transform, &NetEntId), (With<NetworkPlayer>, Without<AttachedAnimation>)>,
    mut animations: Query<(&mut Transform, &AttachedAnimation), Without<NetworkPlayer>>,
) {
    'anim: for (mut anim_transform, animation) in &mut animations {
        for (&p_trans, &net_id) in &players {
            if net_id == animation.0 {
                anim_transform.translation =
                    p_trans.translation + Transform::from_xyz(0.0, 1.0, 0.0).translation;
                anim_transform.rotation = Quat::from_rotation_x(0.0);
                continue 'anim;
            }
        }
    }
}

fn send_heartbeat(
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    let ev = EventToServer::Heartbeat;
    let data = serde_json::to_string(&ev).unwrap();
    event_list_res
        .handler
        .network()
        .send(mse.0, data.as_bytes());
}
