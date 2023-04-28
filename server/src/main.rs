use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent}};
use shared::{event::PlayerConnect, ServerResources, EventFromEndpoint};

fn main() {
    info!("Main Start");
    let mut app = App::new();

    app
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 120.0)))
        .insert_resource(ServerResources::default())
        .add_event::<EventFromEndpoint<PlayerConnect>>()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_startup_system(start_server)
        .add_system(on_player_connect)
        .add_system(shared::tick_server);

    app.run();
}

fn start_server(
    event_list_res: Res<ServerResources>,
) {
    warn!("Start Server");

    let (handler, listener) = node::split::<()>();
    let res_copy = event_list_res.clone();

    std::thread::spawn(move || {
        handler.network().listen(Transport::Udp, "0.0.0.0:3042").unwrap();

        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //let s = from_utf8(data);
                //info!(?s);
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

}

fn on_player_connect(
    mut ev_player_connect: EventReader<EventFromEndpoint<PlayerConnect>>,
) {
    for e in &mut ev_player_connect {
        info!("Got a player connection event {e:?}");
        info!("TODO: Make this send an event to all connected clients");
    }
}
