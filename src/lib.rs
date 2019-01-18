pub mod event;
mod model;
pub mod opt;
pub mod store;

use self::event::{handle_request, NatsPublishActor};
use self::store::Store;
use actix::*;
use futures::prelude::*;
use nitox::{commands::*, NatsClient, NatsError};
use std::rc::Rc;

pub fn initialize(
    future_client: impl Future<Item = NatsClient, Error = NatsError> + 'static,
    subject: String,
) -> impl Future<Item = (), Error = NatsError> + 'static {
    let store: Store = Default::default();
    let addr = store.start();

    future_client.and_then(move |client| {
        let c = Rc::new(client);
        let publish = NatsPublishActor::new(c.clone());
        let publish_addr = Rc::new(publish.start());
        let subscribe_command = SubCommand::builder().subject(subject).build().unwrap();
        c.subscribe(subscribe_command)
            .and_then(move |message_stream| handle_request(message_stream, addr, publish_addr))
    })
}
