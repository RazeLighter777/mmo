use bevy_ecs::prelude::Component;

use crate::hashing;

#[derive(Eq, Hash, PartialEq)]
pub struct ComponentTypeId(u64);

pub trait ComponentDataType: Component {}

pub const fn get_type_id<DataType: 'static + ComponentDataType>() -> ComponentTypeId {
    ComponentTypeId((hashing::string_hash(std::any::type_name::<DataType>())))
}

impl ComponentTypeId {
    pub fn new_with_number(id: u64) -> Self {
        ComponentTypeId(id)
    }
    pub fn new<T: ComponentDataType + 'static>() -> Self {
        let type_id = get_type_id::<T>();
        type_id
    }
}
