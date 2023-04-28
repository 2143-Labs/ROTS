use std::{sync::{Arc, Mutex}, ops::DerefMut};

use bevy::prelude::*;
use message_io::network::Endpoint;
use serde::{Serialize, Deserialize};
pub mod event {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PlayerConnect {
        pub name: String
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GameNetEvent {
    Noop,
    PlayerConnect(event::PlayerConnect),
    PlayerList(Vec<event::PlayerConnect>),
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

#[derive(Resource, Default, Debug, Clone)]
pub struct ServerResources {
    pub event_list: Arc<Mutex<Vec<(Endpoint, GameNetEvent)>>>,
}

pub fn tick_server(
    event_list_res: Res<ServerResources>,
    mut ev_player_connect: EventWriter<EventFromEndpoint<event::PlayerConnect>>,
) {
    let events_to_process = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for event in events_to_process {
        let (_endpoint, e) = event;
        match e {
            GameNetEvent::Noop => warn!("Got noop event"),
            GameNetEvent::PlayerConnect(p) => ev_player_connect.send(EventFromEndpoint::new(_endpoint, p)),
            GameNetEvent::PlayerList(p_list) => ev_player_connect.send_batch(p_list.into_iter().map(|x| EventFromEndpoint::new(_endpoint, x))),
        }
    }
}


