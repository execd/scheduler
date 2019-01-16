use actix::*;
use execd_brain::*;
use futures::{future::ok, prelude::*};
use nitox::{commands::*, NatsClient, NatsClientOptions, NatsError};

fn connect_to_nats() -> impl Future<Item = NatsClient, Error = NatsError> {
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
        .connect_command(connect_cmd)
        .cluster_uri("127.0.0.1:4222")
        .build()
        .unwrap();

    NatsClient::from_options(options)
        .and_then(|client| client.connect())
        .and_then(|client| ok(client))
}

fn handle_request(
    message_stream: impl Stream<Item = nitox::commands::Message, Error = NatsError>,
    addr: Addr<Store>,
) -> impl Future<Item = (), Error = NatsError> {
    message_stream.for_each(move |msg| {
        match serde_json::from_slice::<JobRequest>(&msg.payload) {
            Ok(job_request) => {
                let cl = job_request.clone();
                let res = addr.send(StoreMessage::CreateJob(job_request));
                Arbiter::spawn(res.then(|res| {
                    match res {
                        Ok(r) => println!("Stored {:?}", r),
                        _ => println!("nah!"),
                    }
                    ok(())
                }));
                println!("The req {:?}", cl)
            }
            Err(err) => println!("An err {:?}", err),
        }
        ok(())
    })
}

fn main() {
    let sys = System::new("test");
    let store = Store::new();
    let addr = store.start();
    let tasks = connect_to_nats().and_then(|client| {
        let subscribe_command = SubCommand::builder().subject("messages").build().unwrap();
        client
            .subscribe(subscribe_command)
            .and_then(move |message_stream| handle_request(message_stream, addr))
    });

    Arbiter::spawn(tasks.map_err(|_| ()));
    sys.run();
}
