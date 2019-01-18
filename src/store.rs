use super::model::JobRequest;
use actix::*;
use std::collections::HashMap;

pub enum StoreMessage {
    CreateJob(JobRequest),
}

impl actix::Message for StoreMessage {
    type Result = Option<JobRequest>;
}

#[derive(Default)]
pub struct Store {
    data: HashMap<String, JobRequest>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            data: HashMap::new(),
        }
    }
}

impl Actor for Store {
    type Context = Context<Self>;
}

impl Handler<StoreMessage> for Store {
    type Result = Option<JobRequest>;

    fn handle(&mut self, msg: StoreMessage, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            StoreMessage::CreateJob(req) => self.data.insert(req.ref_id.to_owned(), req.clone()),
        }
    }
}
