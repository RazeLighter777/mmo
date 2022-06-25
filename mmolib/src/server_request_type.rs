use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerRequestType {
    CreateGame {
        world_name: String,
    },
    Login {
        user: String,
        password: String,
    },
    Logout {},
    Join {
        world_name: String,
    },
    Leave {
        world_name: String,
    },
    LoadGame {
        world_name: String,
    },
    SendChat {
        world_name: String,
        message: String,
    },
    RegisterUser {
        user: String,
        password: String,
        invite_code: Option<String>,
    },
    GetUserInviteCode {},
}
