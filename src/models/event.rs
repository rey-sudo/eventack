use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateEventRequest {
    pub topic: String,
    pub user_id: String,
    pub action: String,
    pub value: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Event {
    pub user_id: String,
    pub action: String,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedEvent {
    pub payload: Vec<u8>, // CBOR + LZ4
    pub hash: Vec<u8>,    // SHA-256 hash 
}