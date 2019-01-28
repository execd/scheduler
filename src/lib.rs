pub mod event;
pub mod model;
pub mod opt;
mod store;

use self::event::{handle_request, NatsPublishActor};
use actix::*;
use futures::prelude::*;
use nitox::{commands::*, NatsClient, NatsError};
use std::sync::Arc;

pub fn initialize(
    client: NatsClient,
    subject: String,
) -> impl Future<Item = (), Error = NatsError> + 'static {
    let c = Arc::new(client);
    let publish = NatsPublishActor::new(c.clone());
    let publish_addr = Arc::new(publish.start());
    let subscribe_command = SubCommand::builder().subject(subject).build().unwrap();
    c.subscribe(subscribe_command)
        .and_then(move |message_stream| handle_request(message_stream, publish_addr))
}
