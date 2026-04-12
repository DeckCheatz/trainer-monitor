pub mod handshake;

pub mod proto {
    tonic::include_proto!("trainer");
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Request { id: u64, method: String, params: serde_json::Value },
    Response { id: u64, result: Result<serde_json::Value, String> },
    Notification { method: String, params: serde_json::Value },
}
