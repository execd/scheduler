use serde::*;
use std::collections::HashMap;
use uuid::Uuid;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JobRequest {
    pub id: Uuid,
    pub branch: String,
    pub ref_id: String,
    pub repo_name: String,
    pub repo_url: String,
    pub image: String,
    pub init: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request/response to this application is simple,
/// every response will be in the format of this struct.
#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Response {
    #[serde(rename = "type")]
    pub typ: ResponseType,
    pub value: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponseType {
    Error,
    Message,
}
