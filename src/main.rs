use actix::*;
use execd_brain::event::*;
use execd_brain::initialize;
use execd_brain::opt::Opt;
use futures::prelude::*;
use log::info;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    let config = Opt::from_args();
    let nats_addr = config.nats_addr;
    let subject = config.jobs_subject;

    info!(
        "Listening for job requests at {} on subject {} ...",
        nats_addr, subject
    );

    Arbiter::spawn(initialize(connect_to_nats(nats_addr), subject).map_err(|_| ()));
    System::new("execd-brain").run();
}
