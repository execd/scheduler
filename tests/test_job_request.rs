extern crate execd_brain;
extern crate tokio;

use actix::*;
use execd_brain::event::{connect_to_nats, handle_request, NatsMessage};
use execd_brain::initialize;
use execd_brain::model::{JobRequest, Response, ResponseType};
use futures::Future;
use nitox::commands::Message;
use nitox::NatsError;
use serde_json::Result;
use std::sync::Arc;
use uuid::Uuid;
pub struct FakeNatsActor {}

impl FakeNatsActor {
    pub fn new() -> FakeNatsActor {
        FakeNatsActor {}
    }
}

impl Actor for FakeNatsActor {
    type Context = Context<Self>;
}

pub struct FakeNatsActor2 {}

impl FakeNatsActor2 {
    pub fn new() -> FakeNatsActor2 {
        FakeNatsActor2 {}
    }
}

impl Actor for FakeNatsActor2 {
    type Context = Context<Self>;
}

const JOB_ID: &'static str = "111f6ccf-250d-4ccf-bd05-66848983fe35";
const REPLY_TO: &'static str = "replyto";

#[test]
fn should_send_error_to_rely_to_address_when_job_request_received_was_not_ok() {
    impl Handler<NatsMessage> for FakeNatsActor {
        type Result = ();

        fn handle(&mut self, msg: NatsMessage, _ctx: &mut Context<Self>) -> Self::Result {
            println!("Response {:#?}", msg);
            match msg {
                NatsMessage::ForSubject { subject, message } => {
                    // Assert
                    assert!(subject == REPLY_TO);

                    let maybe_response: Result<Response> = serde_json::from_str(&message);
                    assert!(maybe_response.is_ok());

                    let response = maybe_response.unwrap();
                    assert!(response.typ == ResponseType::Error);

                    let contains_expected = response
                        .value
                        .contains("An error occurred deserializing the message :");
                    assert!(contains_expected);

                    // Not the best, we should timeout the system if possible as well
                    System::current().stop();
                }
            }
        }
    }

    // Arrange
    let msg = Message::builder()
        .subject("test")
        .payload("test")
        .reply_to(Option::from(REPLY_TO.to_owned()))
        .sid("test")
        .build()
        .unwrap();

    let stream = futures::stream::iter_ok::<_, NatsError>(vec![msg]);
    let addr = Arc::new(FakeNatsActor::new().start());

    // Act
    Arbiter::spawn(
        handle_request(stream, addr)
            .map_err(|_| panic!("An error occurred handling the stream...")),
    );

    let sys = System::new("test");
    sys.run();
}

#[test]
fn should_send_message_to_address_when_job_request_received_was_ok() {
    impl Handler<NatsMessage> for FakeNatsActor2 {
        type Result = ();

        fn handle(&mut self, msg: NatsMessage, _ctx: &mut Context<Self>) -> Self::Result {
            println!("Response {:#?}", msg);
            match msg {
                NatsMessage::ForSubject { subject, message } => {
                    // Assert
                    assert!(subject == REPLY_TO);

                    let maybe_response: Result<Response> = serde_json::from_str(&message);
                    assert!(maybe_response.is_ok());

                    let response = maybe_response.unwrap();
                    assert!(response.typ == ResponseType::Message);

                    let contains_expected = response
                        .value
                        .contains(&format!("Job with id {} added to queue.", JOB_ID));
                    assert!(contains_expected);

                    // Not the best, we should timeout the system if possible as well
                    System::current().stop();
                }
            }
        }
    }

    // Arrange
    let uuid = Uuid::parse_str(JOB_ID).unwrap();
    let job = JobRequest {
        id: uuid,
        branch: "test".to_owned(),
        ref_id: "test".to_owned(),
        repo_name: "test".to_owned(),
        repo_url: "test".to_owned(),
        image: "test".to_owned(),
        init: "test".to_owned(),
        metadata: Option::None,
    };
    let json = serde_json::to_string(&job).unwrap();
    let msg = Message::builder()
        .subject("test")
        .payload(json)
        .reply_to(Option::from(REPLY_TO.to_owned()))
        .sid("test")
        .build()
        .unwrap();
    let stream = futures::stream::iter_ok::<_, NatsError>(vec![msg]);
    let addr = Arc::new(FakeNatsActor2::new().start());

    // Act
    Arbiter::spawn(
        handle_request(stream, addr)
            .map_err(|_| panic!("An error occurred handling the stream...")),
    );

    let sys = System::new("test");
    sys.run();
}
