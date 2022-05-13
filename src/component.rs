pub type ComponentId = u64;
pub type ComponentTypeId = u64;
use std::hash::{BuildHasher, Hasher};
use serde::{Serialize, de::DeserializeOwned};
use crate::{entity::EntityId, hashing};
pub const fn get_type_id<DataType: 'static + ComponentDataType>() -> u64 {
    hashing::string_hash(std::any::type_name::<DataType>())
}
pub trait ComponentDataType: Serialize + DeserializeOwned {}


pub trait ComponentInterface : Send + Sync {
    fn get_id(&self) -> ComponentId;
    fn get_type_id(&self) -> ComponentTypeId;
    fn get_parent(&self) -> EntityId;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mutable(&mut self) -> &mut dyn std::any::Any;
}
pub struct Component<T : ComponentDataType> {
    iid : ComponentId,
    pid : EntityId,
    tid : ComponentTypeId,
    data : T
}

impl<T : ComponentDataType + 'static + Send + Sync> ComponentInterface  for Component<T> {
    fn get_id(&self) -> ComponentId {
        self.iid
    }
    fn get_type_id(&self) -> ComponentTypeId {
        self.tid
    }
    fn get_parent(&self) -> EntityId {
        self.pid
    }
    /// Returns an any trait reference
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    /// Returns an any mutable trait reference
    fn as_mutable(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}
impl<T: ComponentDataType + 'static> Component<T> {
    pub fn new(data : T, parent : EntityId) -> Self {
        Self { iid: std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish(), pid: parent, tid: get_type_id::<T>(), data: data }
    }
    pub fn dat(&self) -> &T {
        &self.data
    }
    pub fn dat_mut(&mut self) -> &mut T {
        &mut self.data
    }
}