use std::{collections::HashMap, hash::{BuildHasher, Hasher}};

use crate::component::{self, ComponentDataType};
use crate::world::World;
pub type EntityId = u64;
pub struct Entity {
    iid : EntityId,
    components : HashMap<component::ComponentTypeId,Box<dyn component::ComponentInterface + Send + Sync>>
}
pub struct EntityBuilder {
    e : Entity,
}
impl EntityBuilder {
    pub fn new() -> Self {
        let e = Entity {
            iid : std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish(),
            components : HashMap::new()
        };
        Self { e: e}
    }
    pub fn add<T: component::ComponentDataType + 'static + Send + Sync>(mut self,data : T) -> Self {
        self.e.components.insert(component::get_type_id::<T>(),Box::new(component::Component::new(data, self.e.iid)));
        self
    }
    pub fn build(self) -> Entity {
        self.e
    }
}

impl Entity {
    pub fn get<Q : component::ComponentDataType + 'static>(&self) -> Option<&component::Component<Q>> {
        match self.components.get(&component::get_type_id::<Q>())  {
            Some(component) => {
                component.as_any().downcast_ref::<component::Component<Q>>()
            }
            None => {
                None
            }
        }
    }
    pub fn get_mut<Q : component::ComponentDataType + 'static>(&mut self) -> Option<&mut component::Component<Q>> {
        match self.components.get_mut(&component::get_type_id::<Q>())  {
            Some(component) => {
                component.as_mutable().downcast_mut::<component::Component<Q>>()
            }
            None => {
                None
            }
        }
    }
    pub fn has(&self, tid : component::ComponentTypeId) -> bool {
        self.components.contains_key(&tid)
    }
    pub fn get_by_id(&self, tid : component::ComponentTypeId) -> Option<&Box<dyn component::ComponentInterface + Send + Sync>> {
        self.components.get(&tid)
    }
    pub fn get_all(&self) -> &HashMap<component::ComponentTypeId,Box<dyn component::ComponentInterface + Send + Sync>> {
        &self.components
    }
    pub fn get_id(&self) -> EntityId {
        self.iid
    }
}