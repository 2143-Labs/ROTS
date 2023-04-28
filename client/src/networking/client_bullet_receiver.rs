use bevy::prelude::*;
use message_io::network::Transport;
use shared::GameNetEvent;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_networking_server)
            .add_system(net_tick);
    }
}

fn setup_networking_server() {
    info!("trying_to_start_server");
    let (handler, listener) = message_io::node::split::<()>();

    let (server, _) = match handler.network().connect(Transport::Udp, "127.0.0.1:3042") {
        Ok(d) => d,
        Err(_) => return error!("failed to connect to active server",),
    };

    info!("probably connected");

    let connect_event = GameNetEvent::PlayerConnect(shared::event::PlayerConnect);
    let event_json = serde_json::to_string(&connect_event).unwrap();
    dbg!(&event_json);
    handler.network().send(server, event_json.as_bytes());
    info!("sent json");

    //let mut i = 0;
    //std::thread::spawn(move || {
        //loop {
            //i += 1;
            //h2.network().send(
                //server,
                //&serde_json::to_vec(&NetworkingAction::Heartbeat).unwrap(),
            //);

            ////empty the outs queue because we're using it now
            //let outs = std::mem::replace(&mut *out.lock().unwrap(), Vec::new());

            //for action in outs {
                //let message = serde_json::to_vec(&action).unwrap();

                //h2.network().send(server, &*message);
            //}

            //std::thread::sleep(Duration::from_millis((1000.0f32 / 128.0).floor() as u64));
        //}
    //});

    //listener.for_each(move |event| {
        //match event {
            //NodeEvent::Signal(_s) => {
                //info!("signal...");
            //}
            //NodeEvent::Network(net_event) => match net_event {
                //NetEvent::Message(endpoint, _data) => {
                    ////i += 1;
                    ////handler.network().send(server, &['b' as u8; 1200]);
                    ////println!("got some data", );
                //}
                //NetEvent::Disconnected(_) => {
                    //info!("disconnected from server",);
                //}
                //_ => {}
            //},
        //}
    //});

}

fn net_tick() {
}
