use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Event, Serialize, Deserialize, Debug)]
pub struct ConnectEventClient {
    pub name: Option<String>,
}

#[derive(Event, Serialize, Deserialize, Debug)]
pub struct ConnectEventResp {
    pub your_name: String,
    pub client_id: u64,
}

//impl super::C2S for ConnectEventClient {}

pub struct ClientConnect;

impl super::NEC2S for ClientConnect {
    type ClientSend = ConnectEventClient;
    type ServerResponse = ConnectEventResp;
}
