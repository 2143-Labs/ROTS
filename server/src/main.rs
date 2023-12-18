use std::{ops::DerefMut, sync::{atomic::AtomicI16, Arc}, str::from_utf8};

use rand::{thread_rng, Rng};
use shared::{Config, EventList, ServerNodeHandler, events::ClientRequests};

//use shared::{EventToServer, EventToClient, NetEntId, event::{UpdatePos, ShootBullet, Animation, PlayerDisconnect}, Config};
use bevy::{prelude::*, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent, Endpoint}};
use shared::{event::PlayerInfo, ServerResources, EventFromEndpoint};

const HEARTBEAT_MILLIS: u64 = 200;

fn main() {
    info!("Main Start");
    let mut app = App::new();

    let config = Config::load_from_main_dir();

    start_server(&config, &mut app);

    app
        .add_plugins(LogPlugin::default())
        .insert_resource(config)
        //.insert_resource(EndpointToNetId::default())
        //.insert_resource(HeartbeatList::default())
        //.add_event::<EventFromEndpoint<PlayerInfo>>()
        //.add_event::<UpdatePos>()
        //.add_event::<ShootBullet>()
        //.add_event::<Animation>()
        .add_plugins(MinimalPlugins);
        //.add_systems(Update, (
            //on_player_connect,
            //tick_net_server,
            //send_shooting_to_all_players,
            //send_animations_to_all_players,
            //send_movement_to_all_players,
        //))
        //.add_systems(FixedUpdate, 
            //check_heartbeats
                //.run_if(on_fixed_timer(Duration::from_millis(HEARTBEAT_MILLIS)))
        //);

    app.run();
}

fn start_server(
    config: &Config,
    app: &mut App,
) {
    let config = config.clone();
    let (handler, listener) = node::split::<()>();

    let res = ServerNodeHandler {
        handler: handler.clone(),
    };

    let event_list = EventList::<ClientRequests> {
        event_list: Default::default(),
    };

    app.insert_resource(event_list.clone());
    app.insert_resource(res.clone());

    std::thread::spawn(move || {
        let con_str = (&*config.ip, config.port);
        handler.network().listen(Transport::Tcp, con_str).unwrap();

        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                let data2 = from_utf8(data).unwrap();
                info!("{}", data2);
                match data[0] {
                    b'[' => {
                        let event: Vec<_> = serde_json::from_slice(data).unwrap();

                        let mut elist = event_list.event_list.lock().unwrap();
                        for e in event {
                            elist.push((endpoint, e));
                        }
                    },
                    b'{' | b'"' => {
                        let event = serde_json::from_slice(data).unwrap();
                        info!("{:?}", event);
                        event_list.event_list.lock().unwrap().push((endpoint, event));
                    },
                    d => {
                        info!(d);
                        error!("invalid net req");
                    }
                }
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });
}

