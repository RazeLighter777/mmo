use serde::{Deserialize, Serialize};

use crate::{
    chunk::{Chunk, ChunkId},
    entity_id::EntityId,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerResponseType {
    AuthSuccess {
        session_token: String,
    },
    Ok {},
    AuthFailure {},
    TimedOut {},
    PermissionDenied {},
    Error {
        message: &'static str,
    },
    Ticked {
        world_name: String,
        chunks: Vec<(ChunkId, Chunk)>,
        entities: Vec<(EntityId, Vec<String>)>,
    },
    ChatMessage {
        message: String,
        username: String,
    },
    PlayerList {
        players: Vec<String>,
    }
}
