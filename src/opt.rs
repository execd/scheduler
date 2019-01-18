use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "execd-brain", author = "")]
pub struct Opt {
    /// The address to connect to nats on.
    #[structopt(short = "n", long = "nats-address", default_value = "127.0.0.1:4222")]
    pub nats_addr: String,

    /// The subject to listen for job messages.
    #[structopt(short = "s", long = "jobs-subject", default_value = "execd.jobs")]
    pub jobs_subject: String,
}
