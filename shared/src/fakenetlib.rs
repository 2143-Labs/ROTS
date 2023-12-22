use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Endpoint;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeHandler<T>(T);

pub struct NodeHandlerMockNet;
impl<T> NodeHandler<T> {
    fn network(&self) -> NodeHandlerMockNet {
        NodeHandlerMockNet
    }
}

impl NodeHandlerMockNet {
    fn send(&self, _endpoint: Endpoint, _data: &[u8]) {
    }
}


#[derive(Resource, Clone)]
pub struct ServerResources<T> {
    pub event_list: Arc<Mutex<Vec<(Endpoint, T)>>>,
    pub handler: NodeHandler<()>,
}

#[derive(Resource, Clone)]
pub struct MainServerEndpoint(pub Endpoint);

/// This type is only used for the inital connection, and then it is removed.
#[derive(Resource, Debug)]
pub struct NetworkConnectionTarget {
    pub ip: String,
    pub port: u16,
}

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

pub fn setup_server<T: NetworkingEvent>(commands: Commands, config: Res<NetworkConnectionTarget>) {
    setup_shared::<T>(commands, &config.ip, config.port, true);
}

pub fn setup_client<T: NetworkingEvent>(commands: Commands, config: Res<NetworkConnectionTarget>) {
    setup_shared::<T>(commands, &config.ip, config.port, false);
}

pub fn setup_shared<T: NetworkingEvent>(
    mut commands: Commands,
    ip: &str,
    port: u16,
    is_listener: bool,
) {
    info!(is_listener, "No networking was compiled. Exiting.");
}
