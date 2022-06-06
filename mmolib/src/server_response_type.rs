use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerResponseType {
    AuthSuccess { session_token: String },
    Ok {},
    AuthFailure {},
    TimedOut {},
    PermissionDenied {},
    Ticked {}
}