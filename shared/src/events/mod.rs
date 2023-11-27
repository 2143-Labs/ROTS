pub mod connect;
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

//pub trait C2S {
    //fn 
//}

///Net Event - Client to Server
pub trait NEC2S {
    type ClientSend: Event + Serialize;
    type ServerResponse: Event + DeserializeOwned;
}

fn to_server_event_sender<T>(
    mut events: EventReader<T>,
    //socket info for server
) where T: Event + Serialize {
    for _e in events.read() {
        // serialize to json and write to socket
    }
}

fn from_server_event_receiver<T>(
    mut events: EventWriter<T>,
    //
) where T: Event + DeserializeOwned {
    let json = "";
    match serde_json::from_str(json) {
        Ok(data) => events.send(data),
        Err(_) => {}, // TODO
    }
}

pub fn init_msg_to_server<N: NEC2S>(
    app: &mut App,
){
    app.add_event::<N::ClientSend>();
    app.add_event::<N::ServerResponse>();

    app.add_systems(Update, to_server_event_sender::<N::ClientSend>);
    app.add_systems(Update, from_server_event_receiver::<N::ServerResponse>);

    // add to event checking loop
}
