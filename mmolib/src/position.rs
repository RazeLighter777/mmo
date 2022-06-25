use crate::chunk;
use bevy_ecs::prelude::Component;
use bevy_reflect::Reflect;
use bevy_reflect::ReflectDeserialize;
use serde::{Deserialize, Serialize};
#[derive(Reflect, Default, Serialize, Deserialize, Clone, PartialEq, Debug, Component)]
#[reflect_value(Serialize, PartialEq, Deserialize)]
pub struct Position {
    pub pos: chunk::Position,
    pub load_with_chunk: bool,
}
