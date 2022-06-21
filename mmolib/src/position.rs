use bevy_ecs::prelude::Component;
use bevy_reflect::{Reflect};
use serde::{Deserialize, Serialize};
use bevy_reflect::ReflectDeserialize;
use crate::chunk;
#[derive(Reflect, Default, Serialize, Deserialize, Clone, PartialEq, Debug,Component)]
#[reflect_value(Serialize, PartialEq, Deserialize)]
pub struct Position {
    pub pos : chunk::Position,
    pub load_with_chunk : bool
}