pub struct ConnectEventClient {
    pub name: Option<String>,
}

pub struct ConnectEventResp {
    pub your_name: String,
    pub client_id: u64,
}

pub trait NEC2S {
    type ClientSend;
    type ServerResponse;
}

pub struct ClientConnect;

impl NEC2S for ClientConnect {
    type ClientSend = ConnectEventClient;
    type ServerResponse = ConnectEventResp;
}
