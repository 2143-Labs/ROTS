use bevy::prelude::*;
use message_io::{
    network::{Endpoint, NetEvent, Transport},
    node::{NodeEvent, NodeHandler},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Resource, Clone)]
pub struct ServerResources<T> {
    pub event_list: Arc<Mutex<Vec<(Endpoint, T)>>>,
    pub handler: NodeHandler<()>,
}

#[derive(Resource, Clone)]
pub struct MainServerEndpoint(pub Endpoint);

pub use crate::event::client::EventToClient;
pub use crate::event::server::EventToServer;

pub trait NetworkingEvent:
    Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static + core::fmt::Debug
{
}
impl NetworkingEvent for EventToServer {}
impl NetworkingEvent for EventToClient {}

pub fn send_event_to_server<T: NetworkingEvent>(
    handler: &NodeHandler<()>,
    endpoint: Endpoint,
    event: &T,
) {
    handler
        .network()
        .send(endpoint, &postcard::to_stdvec(&event).unwrap());
}

pub fn setup_server<T: NetworkingEvent>(commands: Commands, config: Res<crate::Config>) {
    setup_shared::<T>(commands, config, true);
}

pub fn setup_client<T: NetworkingEvent>(commands: Commands, config: Res<crate::Config>) {
    setup_shared::<T>(commands, config, false);
}

pub fn setup_shared<T: NetworkingEvent>(
    mut commands: Commands,
    config: Res<crate::Config>,
    is_listener: bool,
) {
    info!(is_listener, "Seting up networking!");

    let (handler, listener) = message_io::node::split::<()>();

    let res = ServerResources::<T> {
        handler: handler.clone(),
        event_list: Default::default(),
    };
    commands.insert_resource(res.clone());
    info!(
        "Setup server resources for {}",
        std::any::type_name::<ServerResources::<T>>()
    );

    let con_str = (&*config.ip, config.port);
    if is_listener {
        let (_, addr) = handler.network().listen(Transport::Udp, con_str).unwrap();
        info!(?addr, "Listening")
    } else {
        let (endpoint, addr) = handler.network().connect(Transport::Udp, con_str).unwrap();
        commands.insert_resource(MainServerEndpoint(endpoint));
        info!(?addr, "Connected");
    }

    std::thread::spawn(move || {
        listener.for_each(|event| on_node_event(&res, event));
    });
}

pub fn on_node_event<T: NetworkingEvent>(res: &ServerResources<T>, event: NodeEvent<'_, ()>) {
    let net_event = match event {
        NodeEvent::Network(n) => n,
        NodeEvent::Signal(_) => {
            panic!("MESSAGE SERVER SHUTDOWN SIGNAL RECEIVED!!!");
            // TODO graceful shutdown
        }
    };

    match net_event {
        NetEvent::Connected(_, _) => info!("Network Connected"),
        NetEvent::Accepted(_endpoint, _listener) => info!("Connection Accepted"),
        NetEvent::Message(endpoint, data) => {
            info!(?data, "res");
            let event = match postcard::from_bytes(data) {
                Ok(e) => e,
                Err(_) => {
                    warn!(?endpoint, "Got invalid json from endpoint");
                    return;
                }
            };
            let pair = (endpoint, event);

            res.event_list.lock().unwrap().push(pair);
        }
        NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
    }
}
