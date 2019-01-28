use actix::*;
use futures::{future, prelude::*};
use nitox::{commands::*, NatsClient};
use std::sync::Arc;

// create actor for nats messages
#[derive(Debug)]
pub enum NatsMessage {
    ForSubject { subject: String, message: String },
}

impl actix::Message for NatsMessage {
    type Result = ();
}

pub struct NatsPublishActor {
    client: Arc<NatsClient>,
}

impl NatsPublishActor {
    pub fn new(client: Arc<NatsClient>) -> NatsPublishActor {
        NatsPublishActor { client }
    }
}

impl Actor for NatsPublishActor {
    type Context = Context<Self>;
}

impl Handler<NatsMessage> for NatsPublishActor {
    type Result = ();

    fn handle(&mut self, msg: NatsMessage, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            NatsMessage::ForSubject { subject, message } => {
                let publish_command = PubCommand::builder()
                    .subject(subject)
                    .payload(message)
                    .build()
                    .unwrap();
                let _ = self
                    .client
                    .publish(publish_command)
                    .and_then(|_| future::ok(()));
            }
        }
    }
}
