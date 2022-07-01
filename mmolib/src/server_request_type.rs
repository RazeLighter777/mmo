use serde::{Deserialize, Serialize};

use crate::entity_id::EntityId;
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerRequestType {
    CreateGame {
        world_name: String,
    },
    PlayerList {
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
    Spawn {
        world_name: String,
        player_parameters: String,
    },
    RegisterUser {
        user: String,
        password: String,
        invite_code: Option<String>,
    },
    GetUserInviteCode {},
    PlayerAction {
        world_name: String,
        action: PlayerActionType,
    },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Northeast,
    Southeast,
    Southwest,
    Northwest,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PlayerActionType {
    Move(Direction),
    Attack(EntityId),
    UseOn { item: EntityId, target: EntityId },
    Pickup(EntityId),
    Drop(EntityId),
}
