use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerResponseType {
    AuthSuccess { session_token: String },
    Ok {},
    AuthFailure {},
    TimedOut {},
    PermissionDenied {},
    Error { message: &'static str },
    Ticked { world_name: String },
    ChatMessage { message: String, username: String },
}
