use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Attributes {
    project_id: String,
    cluster_name: String,
    cluster_location: String,
    type_url: String,
    payload: Value,
}
