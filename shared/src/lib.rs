use std::{sync::{Arc, Mutex}, ops::DerefMut};

use bevy::prelude::*;
use message_io::{network::Endpoint, node::NodeHandler};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct NetEntId(pub u64);

pub mod event {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerInfo {
        pub name: String,
        pub id: NetEntId,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToClient {
    Noop,
    PlayerConnect(event::PlayerInfo),
    PlayerList(Vec<event::PlayerInfo>),
    UpdatePos(NetEntId, Transform),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventToServer {
    Noop,
    Connect{name: String},
    UpdatePos(Transform),
}

#[derive(Debug, Clone)]
pub struct EventFromEndpoint<E> {
    pub event: E,
    pub endpoint: Endpoint,
}

impl<E> EventFromEndpoint<E> {
    pub fn new(endpoint: Endpoint, e: E) -> Self {
        EventFromEndpoint { event: e, endpoint }
    }
}

#[derive(Resource, Clone)]
pub struct ServerResources<T> {
    pub event_list: Arc<Mutex<Vec<(Endpoint, T)>>>,
    pub handler: NodeHandler<()>,
}

//pub fn tick_net_client(
    //event_list_res: Res<ServerResources<EventToClient>>,
    //mut ev_player_connect: EventWriter<EventFromEndpoint<event::PlayerInfo>>,
    //mut ev_player_movement: EventWriter<EventFromEndpoint<event::PlayerInfo>>,
//) {
    //let events_to_process = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    //for event in events_to_process {
        //let (_endpoint, e) = event;
        //match e {
            //EventToClient::Noop => warn!("Got noop event"),
            //EventToClient::PlayerConnect(p) => ev_player_connect.send(EventFromEndpoint::new(_endpoint, p)),
            //EventToClient::PlayerList(p_list) => ev_player_connect.send_batch(p_list.into_iter().map(|x| EventFromEndpoint::new(_endpoint, x))),
        //}
    //}
//}


