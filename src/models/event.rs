use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub topic: String,
    pub payload: Value,
}
