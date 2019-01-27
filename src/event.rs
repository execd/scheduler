extern crate nitox;
extern crate serde;
extern crate serde_json;
use super::model::{JobRequest, Response, ResponseType};
use actix::*;
use futures::{future::ok, prelude::*};
use log::error;
use nitox::{commands::*, NatsClient, NatsClientOptions, NatsError};
use std::sync::Arc;
use uuid::Uuid;

// For messages that are request/reply but have no reply to,
// or the reply to cannot be decoded, they get sent to this subject
const BLACK_HOLE_SUBJECT: &str = "blackhole";

// create actor for nats messages
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
                let _ = self.client.publish(publish_command).and_then(|_| ok(()));
            }
        }
    }
}

pub fn connect_to_nats(nats_addr: String) -> impl Future<Item = NatsClient, Error = NatsError> {
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
        .connect_command(connect_cmd)
        .cluster_uri(nats_addr.to_owned())
        .build()
        .unwrap();

    NatsClient::from_options(options)
        .and_then(|client| client.connect())
        .and_then(ok)
        .map_err(move |e| panic!("Failed to connect to {} ! Cause: {}", nats_addr, e))
}

pub fn handle_request(
    message_stream: impl Stream<Item = nitox::commands::Message, Error = NatsError> + 'static,
    publish_addr: Arc<Addr<NatsPublishActor>>,
) -> impl Future<Item = (), Error = NatsError> + 'static {
    message_stream.for_each(move |msg| {
        decode_message(msg, &publish_addr);
        ok(())
    })
}

fn decode_message(msg: nitox::commands::Message, publish_addr: &Arc<Addr<NatsPublishActor>>) {
    let reply_to = msg
        .reply_to
        .unwrap_or_else(|| BLACK_HOLE_SUBJECT.to_owned());
    match serde_json::from_slice::<JobRequest>(&msg.payload) {
        Ok(job_request) => {
            let job_id = job_request.id;
            let publish = publish_addr.clone();
            respond(&reply_to, &publish, job_id);
        }
        Err(err) => {
            let response = Response {
                typ: ResponseType::Error,
                value: format!("An error occurred deserializing the message : {}", err),
            };
            publish_addr.do_send(NatsMessage::ForSubject {
                subject: reply_to.to_owned(),
                message: serde_json::to_string(&response).unwrap(),
            })
        }
    }
}

fn respond(reply_to: &str, publish_addr: &Arc<Addr<NatsPublishActor>>, job_id: Uuid) {
    let response = Response {
        typ: ResponseType::Message,
        value: format!("Job with id {} added to queue.", job_id),
    };
    publish_addr.do_send(NatsMessage::ForSubject {
        subject: reply_to.to_owned(),
        message: serde_json::to_string(&response).unwrap(),
    })
}
