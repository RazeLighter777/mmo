use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    sync::Arc,
};

//use crate::world::World;
use crate::{
    component::{self, ComponentDataType},
    registry::Registry, world::World,
};
pub type EntityId = u64;
pub struct Entity {
    iid: EntityId,
    components: HashMap<component::ComponentTypeId, Box<dyn component::ComponentInterface>>,
}
pub struct EntityBuilder<'a> {
    e: Entity,
    world : &'a World,
}
impl<'a> EntityBuilder<'a> {
    pub fn new_with_id(id: EntityId, world : &'a World) -> EntityBuilder<'a> {
        let e = Entity {
            iid: id,
            components: HashMap::new(),
        };
        Self { e, world : world}
    }
    pub fn new(registry: &Registry, world : &'a World) -> EntityBuilder<'a> {
        Self::new_with_id(
            std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish(),
            world,
        )
    }
    pub fn add<T: component::ComponentDataType + 'static + Send + Sync>(mut self, data: T) -> Self {
        let cmps = component::Component::new(data, self.e.iid, self.world);
        for boxcmp in cmps {
            self.e.components.insert(boxcmp.get_type_id(), boxcmp);
        }
        self
    }
    pub fn add_existing(mut self, mut component: Box<dyn component::ComponentInterface>) -> Self {
        component.set_parent(self.e.get_id());
        self.e.components.insert(component.get_type_id(), component);
        self
    }
    pub fn build(self) -> Entity {
        self.e
    }
}

impl Entity {
    pub fn get<Q: component::ComponentDataType + 'static>(
        &self,
    ) -> Option<&component::Component<Q>> {
        match self.components.get(&component::get_type_id::<Q>()) {
            Some(component) => component.as_any().downcast_ref::<component::Component<Q>>(),
            None => None,
        }
    }
    pub fn get_mut<Q: component::ComponentDataType + 'static>(
        &mut self,
    ) -> Option<&mut component::Component<Q>> {
        match self.components.get_mut(&component::get_type_id::<Q>()) {
            Some(component) => component
                .as_mutable()
                .downcast_mut::<component::Component<Q>>(),
            None => None,
        }
    }
    pub fn has(&self, tid: component::ComponentTypeId) -> bool {
        self.components.contains_key(&tid)
    }
    pub fn get_by_id(
        &self,
        tid: component::ComponentTypeId,
    ) -> Option<&Box<dyn component::ComponentInterface>> {
        self.components.get(&tid)
    }
    pub fn get_all(
        &self,
    ) -> &HashMap<component::ComponentTypeId, Box<dyn component::ComponentInterface>> {
        &self.components
    }
    pub fn get_id(&self) -> EntityId {
        self.iid
    }
}
