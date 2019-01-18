extern crate nitox;
extern crate serde;
extern crate serde_json;
use super::model::{JobRequest, Response, ResponseType};
use super::store::{Store, StoreMessage};
use actix::*;
use futures::{future::ok, prelude::*};
use log::error;
use nitox::{commands::*, NatsClient, NatsClientOptions, NatsError};
use std::rc::Rc;
use uuid::Uuid;

// For messages that are request/reply but have no reply to,
// or the reply to cannot be decoded, they get sent to this subject
const BLACK_HOLE_SUBJECT: &str = "blackhole";

pub trait ExecdNatsClient {
    fn publish(&self, cmd: PubCommand) -> Box<Future<Item = (), Error = NatsError> + Send + Sync>;
}

impl ExecdNatsClient for NatsClient {
    fn publish(&self, cmd: PubCommand) -> Box<Future<Item = (), Error = NatsError> + Send + Sync> {
        Box::from(self.publish(cmd))
    }
}

// create actor for nats messages
pub enum NatsMessage {
    ForSubject { subject: String, message: String },
}

impl actix::Message for NatsMessage {
    type Result = ();
}

pub struct NatsPublishActor {
    client: Rc<ExecdNatsClient>,
}

impl NatsPublishActor {
    pub fn new(client: Rc<ExecdNatsClient>) -> NatsPublishActor {
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
                    .clone()
                    .publish(publish_command)
                    .and_then(|_| ok(()));
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
    store_addr: Addr<Store>,
    publish_addr: Rc<Addr<NatsPublishActor>>,
) -> impl Future<Item = (), Error = NatsError> + 'static {
    message_stream.for_each(move |msg| {
        decode_message(msg, &store_addr, &publish_addr);
        ok(())
    })
}

fn decode_message(
    msg: nitox::commands::Message,
    store_addr: &Addr<Store>,
    publish_addr: &Rc<Addr<NatsPublishActor>>,
) {
    let reply_to = msg
        .reply_to
        .unwrap_or_else(|| BLACK_HOLE_SUBJECT.to_owned());
    match serde_json::from_slice::<JobRequest>(&msg.payload) {
        Ok(job_request) => {
            let job_id = job_request.id;
            let res = store_addr.send(StoreMessage::CreateJob(job_request));
            let publish = publish_addr.clone();
            Arbiter::spawn(res.then(move |res| {
                match res {
                    Ok(r) => respond(&reply_to, &publish, r, job_id),
                    _ => error!("An error occured when storing the job (id {})!", job_id),
                }
                ok(())
            }));
        }
        Err(err) => error!("An error occurred: {:?}", err),
    }
}

fn respond(
    reply_to: &str,
    publish_addr: &Rc<Addr<NatsPublishActor>>,
    maybe_job_request: Option<JobRequest>,
    job_id: Uuid,
) {
    match maybe_job_request {
        Some(job) => {
            let response = Response {
                typ: ResponseType::Error,
                value: format!(
                    "The job with id {} was overwritten! \
                    This can only happen if the job ID for this request was sent in a historical request.",
                    job.id
                ),
            };
            publish_addr.do_send(NatsMessage::ForSubject {
                subject: reply_to.to_owned(),
                message: serde_json::to_string(&response).unwrap(),
            })
        }
        None => {
            let response = Response {
                typ: ResponseType::Message,
                value: format!("Job with id {} added to queue.", job_id),
            };
            publish_addr.do_send(NatsMessage::ForSubject {
                subject: reply_to.to_owned(),
                message: serde_json::to_string(&response).unwrap(),
            })
        }
    }
}
