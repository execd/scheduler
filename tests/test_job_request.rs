extern crate execd_brain;
extern crate tokio;

use actix::*;
use execd_brain::event::connect_to_nats;
use execd_brain::initialize;
use execd_brain::model::JobRequest;
use futures::Future;
use uuid::Uuid;

// Note these tests currently require a running nats instance.

#[test]
fn should_return_message_when_job_request_is_correct() {
    let port = 4222;
    let subject = "should_return_message_when_job_request_is_correct";
    let job = JobRequest {
        id: Uuid::new_v4(),
        branch: "test".to_owned(),
        ref_id: "test".to_owned(),
        repo_name: "test".to_owned(),
        repo_url: "test".to_owned(),
        image: "test".to_owned(),
        init: "test".to_owned(),
        metadata: Option::None,
    };

    let payload = serde_json::to_string(&job).unwrap();

    let client = connect_to_nats(format!("127.0.0.1:{}", port).to_owned())
        .map_err(|_| println!("Failed to connect..."))
        .and_then(move |client| {
            client
                .request(subject.into(), payload.into())
                .map_err(|_| panic!("Request timed out!"))
                .map(|msg| {
                    let job = serde_json::from_slice::<execd_brain::model::Response>(&msg.payload);
                    assert!(job.is_ok());
                    assert!(job.unwrap().typ == execd_brain::model::ResponseType::Message);
                    System::current().stop();
                })
        });
    Arbiter::spawn(
        connect_to_nats("127.0.0.1:4222".into())
            .map_err(|_| ())
            .and_then(move |client| initialize(client, subject.into()).map_err(|_| ())),
    );
    Arbiter::spawn(client);
    System::new("test").run();
}

#[test]
fn should_return_error_when_job_request_is_not_correct() {
    let port = 4222;
    let subject = "should_return_error_when_job_request_is_not_correct";

    let payload = serde_json::to_string("string").unwrap();

    let client = connect_to_nats(format!("127.0.0.1:{}", port).to_owned())
        .map_err(|_| println!("Failed to connect..."))
        .and_then(move |client| {
            client
                .request(subject.into(), payload.into())
                .map_err(|_| panic!("Request timed out!"))
                .map(|msg| {
                    let job = serde_json::from_slice::<execd_brain::model::Response>(&msg.payload);
                    assert!(job.is_ok());
                    assert!(job.unwrap().typ == execd_brain::model::ResponseType::Error);
                    System::current().stop();
                })
        });
    Arbiter::spawn(
        connect_to_nats("127.0.0.1:4222".into())
            .map_err(|_| ())
            .and_then(move |client| initialize(client, subject.into()).map_err(|_| ())),
    );
    Arbiter::spawn(client);
    System::new("test").run();
}
// fn with_nats<T>(port: u16, test: T)
// where
//     T: FnOnce(u16) -> () + UnwindSafe,
// {
//     // let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
//     // let socket = TcpListener::bind(&addr).unwrap();
//     // println!("Will listen on {}", port);
//     // let mut runtime = tokio::runtime::Runtime::new().unwrap();
//     // runtime.spawn(
//     //     socket
//     //         .incoming()
//     //         .map_err(|e| println!("failed to accept socket; error = {:?}", e))
//     //         .for_each(move |socket| process(socket)),
//     // );

//     // test(port);

//     // thread::spawn(move || {
//     //     let sys = System::new("test");
//     //     sys.run();
//     // });
//     // let (tx, rx) = oneshot::channel();
//     // runtime.spawn(f.then(|r| tx.send(r).map_err(|_| panic!("Cannot send Result"))));
//     // let connection_result = rx.wait().expect("Cannot wait for a result");
//     // let _ = runtime.shutdown_now().wait();
// }
// // socket
// //         .incoming()
// //         .map_err(|e| println!("failed to accept socket; error = {:?}", e))
// //         .for_each(move |socket| process(socket)),
// fn process(
//     socket: TcpStream,
//     sids: Arc<Mutex<HashMap<String, String>>>,
// ) -> impl Future<Item = (), Error = ()> {
//     let (sink, stream) = OpCodec::default().framed(socket).split();

//     let server_info = Op::INFO(
//         ServerInfo::builder()
//             .server_id("nats-test")
//             .version(::std::env::var("CARGO_PKG_VERSION").unwrap())
//             .go("lol")
//             .host("127.0.0.1")
//             .port(4222u32)
//             .max_payload(::std::u32::MAX)
//             .build()
//             .unwrap(),
//     );

//     sink.send(server_info)
//         .and_then(|sink| {
//             stream
//                 .from_err()
//                 .and_then(move |op| process_op(op, sids.clone()))
//                 .filter(|maybe_op| maybe_op.is_some())
//                 .and_then(|op| {
//                     let o = op.unwrap();
//                     println!("Response {:?}", o);
//                     ok(o)
//                 })
//                 .forward(sink)
//         })
//         .map(|_| ())
//         .map_err(|e| println!("Socket error... {:#?}", e))
// }

// fn process_op(
//     op: Op,
//     sids: Arc<Mutex<HashMap<String, String>>>,
// ) -> impl Future<Item = Option<Op>, Error = NatsError> {
//     println!("Got an op: {:?}", op);
//     let op = match op {
//         Op::PING => Option::from(Op::PONG),
//         Op::PONG => Option::None,
//         Op::SUB(cmd) => {
//             println!("setting SID {:#?} for subject {:#?}", cmd.sid, cmd.subject);
//             sids.lock().unwrap().insert(cmd.subject, cmd.sid);
//             Option::from(Op::OK)
//         }
//         Op::PUB(cmd) => {
//             let payload = std::str::from_utf8(&cmd.payload).unwrap();
//             if payload == "SHUTDOWN" {
//                 System::current().stop();
//                 Option::None
//             } else {
//                 let mut builder = Message::builder();
//                 let sub = cmd.subject.clone();
//                 let r = cmd.reply_to.unwrap_or(sub);
//                 println!("Will respond to {:#?} ", r);
//                 builder.subject(r);
//                 {
//                     let sids = sids.lock().unwrap();
//                     let default = &"".to_owned();
//                     let sid = sids.get(&cmd.subject).unwrap_or(default);
//                     println!("Got sid {:#?} for subject {:#?}", sid, cmd.subject);
//                     builder.sid((*sid).clone());
//                 }
//                 builder.payload(cmd.payload);
//                 let msg = builder.build().unwrap();
//                 Option::from(Op::MSG(msg))
//             }
//         }
//         Op::UNSUB(cmd) => Option::None,
//         Op::CONNECT(_) => Option::None,
//         _ => panic!("Unexpected op!"),
//     };
//     ok(op)
// }
