use std::fmt::Display;

use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::FromWorld;
use bevy_reflect::Reflect;
use bevy_reflect::ReflectDeserialize;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(
    Reflect, Default, Serialize, Deserialize, Clone, PartialEq, Debug, Component, Eq, Hash, Copy,
)]
#[reflect_value(Serialize, PartialEq, Deserialize)]
pub struct EntityId(u64);

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entity<{:X}>", self.0)
    }
}

impl EntityId {
    pub fn new_with_number(id: u64) -> Self {
        EntityId(id)
    }
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        EntityId(rng.gen())
    }
    pub fn id(&self) -> u64 {
        self.0
    }
}
