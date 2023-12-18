pub mod connect;
use bevy::prelude::*;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub enum ClientRequests {
    Connect(connect::Req),
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub enum ServerResponses {
    Connect(connect::Res),
}

///Net Event - Client to Server
pub trait NEC2S {
    type ClientSend: Event + Serialize;
    type ServerResponse: Event + DeserializeOwned;
}

pub trait NamedEvent {
    fn name() -> &'static str;
}
