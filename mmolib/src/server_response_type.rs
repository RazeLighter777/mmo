use serde::{Deserialize, Serialize};

use crate::{
    block_type::BlockTypeId,
    chunk::{Chunk, ChunkId, Position},
    component::ComponentTypeId,
    entity_id::EntityId,
};

pub type EncodingType = serde_json::Value;

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
        component_updates: Vec<ComponentUpdate>,
        block_updates: Vec<BlockUpdate>,
    },

    ChatMessage {
        message: String,
        username: String,
    },
    PlayerList {
        players: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub struct ComponentUpdate {
    entity_id: EntityId,
    component_type_id: ComponentTypeId,
    component_update_type: ComponentUpdateType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ComponentUpdateType {
    Removed,
    Added { packet: EncodingType },
    Changed { packet: EncodingType },
}

impl ComponentUpdate {
    pub fn new(
        entity: EntityId,
        copmonent_type: ComponentTypeId,
        ctype: ComponentUpdateType,
    ) -> Self {
        ComponentUpdate {
            entity_id: entity,
            component_type_id: copmonent_type,
            component_update_type: ctype,
        }
    }
    pub fn get_entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub fn get_component_type_id(&self) -> ComponentTypeId {
        self.component_type_id
    }
    pub fn get_component_update_info(&self) -> &ComponentUpdateType {
        &self.component_update_type
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct BlockUpdate {
    pub block_pos: Position,
    pub block_type_id: BlockTypeId,
}
