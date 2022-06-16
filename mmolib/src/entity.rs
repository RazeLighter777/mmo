use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    sync::Arc,
};

//use crate::world::World;
use crate::{
    component::{self, ComponentDataType},
    registry::Registry,
    world::World,
};
pub type EntityId = u64;
pub struct Entity {
    iid: EntityId,
    components: HashMap<component::ComponentTypeId, component::ComponentId>,
}
pub struct EntityBuilder<'a> {
    e: Entity,
    world: &'a World,
    components: Vec<Box<dyn component::ComponentInterface>>,
}
impl<'a> EntityBuilder<'a> {
    pub fn new_with_id(id: EntityId, world: &'a World) -> EntityBuilder<'a> {
        let e = Entity {
            iid: id,
            components: HashMap::new(),
        };
        Self {
            e,
            world: world,
            components: Vec::new(),
        }
    }
    pub fn new(registry: &Registry, world: &'a World) -> EntityBuilder<'a> {
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
            self.e
                .components
                .insert(boxcmp.get_type_id(), boxcmp.get_id());
            self.components.push(boxcmp);
        }
        self
    }
    pub fn add_existing(mut self, mut component: Box<dyn component::ComponentInterface>) -> Self {
        component.set_parent(self.e.get_id());
        self.e
            .components
            .insert(component.get_type_id(), component.get_id());
        self
    }
    pub fn build(self) -> (Vec<Box<dyn component::ComponentInterface>>, Entity) {
        (self.components, self.e)
    }
}

impl Entity {
    pub fn has(&self, tid: component::ComponentTypeId) -> bool {
        self.components.contains_key(&tid)
    }
    pub fn remove(&mut self, tid: component::ComponentTypeId) -> bool {
        self.components.remove(&tid).is_some()
    }
    pub fn get_id(&self) -> EntityId {
        self.iid
    }
    pub fn get(&self, tid: component::ComponentTypeId) -> Option<component::ComponentId> {
        self.components.get(&tid).map(ToOwned::to_owned)
    }
    pub fn get_assured(&self, tid: component::ComponentTypeId) -> component::ComponentId {
        self.components
            .get(&tid)
            .map(ToOwned::to_owned)
            .expect("Entity did not have component of correct type in call to get_assured")
    }
    pub fn get_component_ids(&self) -> Vec<component::ComponentId> {
        self.components.values().cloned().collect()
    }
}
