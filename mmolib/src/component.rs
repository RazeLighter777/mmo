use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::hashing;

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct ComponentTypeId(u64);

pub const fn get_type_id<DataType: 'static>() -> ComponentTypeId {
    ComponentTypeId((hashing::string_hash(std::any::type_name::<DataType>())))
}

pub const fn get_type_id_from_str(s: &str) -> ComponentTypeId {
    ComponentTypeId((hashing::string_hash(s)))
}

impl ComponentTypeId {
    pub fn new_with_number(id: u64) -> Self {
        ComponentTypeId(id)
    }
    pub fn new<T: 'static>() -> Self {
        let type_id = get_type_id::<T>();
        type_id
    }
}
