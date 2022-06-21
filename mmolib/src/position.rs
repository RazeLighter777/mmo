use bevy_ecs::prelude::Component;
use serde::Deserialize;

use crate::chunk;

#[derive(Component, Deserialize)]
pub struct Position {
    pub pos : chunk::Position,
    pub load_with_chunk : bool
}