use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerRequestType {
    CreateGame { world_name: String },
    Login { user: String, password: String },
    Logout {},
    Join { world_name : String },
}
