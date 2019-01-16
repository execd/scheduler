use actix::*;
use serde::*;
use std::collections::HashMap;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct JobRequest {
    branch: String,
    ref_id: String,
    repo_name: String,
    repo_url: String,
    image: String,
    init: String,
    metadata: Option<HashMap<String, String>>,
}

pub enum StoreMessage {
    CreateJob(JobRequest),
    GetJob(String),
}

impl actix::Message for StoreMessage {
    type Result = Option<JobRequest>;
}

pub struct Store {
    data: HashMap<String, JobRequest>,
}

impl Store {
    pub fn new() -> Store{
        Store { data : HashMap::new() }
    }
}

impl Actor for Store {
    type Context = Context<Self>;
}

impl Handler<StoreMessage> for Store {
    type Result = Option<JobRequest>;

    fn handle(&mut self, msg: StoreMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            StoreMessage::CreateJob(req) => {
                self.data.insert(req.ref_id.to_owned(), req.clone());
                Option::from(req)
            }
            StoreMessage::GetJob(id) => self.data.get(&id).map(|job_request| job_request.clone()),
        }
    }
}
