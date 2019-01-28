use super::model::{JobRequest, Response, ResponseType};
use actix::*;
use futures::{future::ok, prelude::*};
use nitox::{commands::*, NatsClient, NatsClientOptions, NatsError};
use redis::{Client, Commands};
use std::sync::Arc;
use uuid::Uuid;

// create actor for nats messages
pub enum RedisActorMessage {
    PUSH(JobRequest),
    POP,
}

impl actix::Message for RedisActorMessage {
    type Result = ();
}

pub struct RedisActor {
    client: Arc<Client>,
}

impl RedisActor {
    pub fn new(client: Arc<Client>) -> RedisActor {
        RedisActor { client }
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

impl Handler<RedisActorMessage> for RedisActor {
    type Result = ();

    fn handle(&mut self, msg: RedisActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let f = self.client.get_connection().unwrap(); 
        
    }
}
