
#[derive(Eq, Hash, PartialEq, Copy, Clone, Deserialize,PartialOrd, Ord, Debug)]
pub struct ComponentId(pub u64);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Deserialize, PartialOrd, Ord, Debug)]
pub struct ComponentTypeId(pub u64);
use crate::{entity::EntityId, hashing, registry, world::World};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::Value;
use std::{
    hash::{BuildHasher, Hasher},
    sync::Arc, fmt::Debug,
};
pub const fn get_type_id<DataType: 'static + ComponentDataType>() -> ComponentTypeId {
    ComponentTypeId((hashing::string_hash(std::any::type_name::<DataType>())))
}
pub trait ComponentDataType: Serialize + DeserializeOwned + Sync + Send {
    fn post_deserialization(&mut self, world: &World) -> Vec<Box<dyn ComponentInterface>> {
        Vec::new()
    }
}

pub trait ComponentInterface: Send + Sync + Debug {
    fn get_id(&self) -> ComponentId;
    fn get_type_id(&self) -> ComponentTypeId;
    fn get_parent(&self) -> EntityId;
    fn as_any(&self) -> &dyn std::any::Any;
    fn set_parent(&mut self, pid: EntityId);
    fn as_mutable(&mut self) -> &mut dyn std::any::Any;
    fn get_json(&self) -> Value;
}

#[derive(Debug)]
pub struct Component<T: ComponentDataType> {
    iid: ComponentId,
    pid: EntityId,
    tid: ComponentTypeId,
    data: T,
}

impl<T: ComponentDataType + 'static + Send + Sync + Debug> ComponentInterface for Component<T> {
    fn get_id(&self) -> ComponentId {
        self.iid
    }
    fn get_type_id(&self) -> ComponentTypeId {
        self.tid
    }
    fn get_parent(&self) -> EntityId {
        self.pid
    }
    fn set_parent(&mut self, pid: EntityId) {
        self.pid = pid;
    }
    /// Returns an any trait reference
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    /// Returns an any mutable trait reference
    fn as_mutable(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn get_json(&self) -> Value {
        serde_json::to_value(self.dat()).expect("Could not serialize component")
    }
}
impl<T: ComponentDataType + 'static + Debug> Component<T> {
    pub fn new(mut data: T, parent: EntityId, world: &World) -> Vec<Box<dyn ComponentInterface>> {
        let mut res = data.post_deserialization(world);
        let main = Self {
            iid: ComponentId((std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish())),
            pid: parent,
            tid: get_type_id::<T>(),
            data: data,
        };
        res.push(Box::new(main));
        res
    }
    pub fn dat(&self) -> &T {
        &self.data
    }
    pub fn dat_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
