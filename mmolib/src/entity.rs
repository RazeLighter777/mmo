use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::{BuildHasher, Hasher},
    sync::Arc,
};

use bevy_ecs::prelude::Component;
use serde::Deserialize;

//use crate::world::World;
use crate::{game_world::GameWorld, registry::Registry};
use rand::Rng;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Deserialize, Debug, Component)]
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
