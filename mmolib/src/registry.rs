use std::sync::Arc;
use std::{collections::HashMap, fmt};

use crate::block_type::BlockType;
use crate::component::{get_type_id, get_type_id_from_str, ComponentTypeId};
use crate::entity_id::EntityId;
use crate::game_world::GameWorld;
use crate::raws::Raw;
use crate::server_response_type::{ComponentUpdate, ComponentUpdateType};
use crate::uuid_map::UuidMap;
use crate::{
    block_type,
    component::{self},
    entity_id,
    raws::RawTree,
};
use crate::{hashing, player, position};
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::{Component, Entity, ReflectComponent};
use bevy_ecs::query::{ChangeTrackers, Changed};
use bevy_ecs::world::{EntityMut, EntityRef, World};
use bevy_reflect::serde::{ReflectDeserializer, ReflectSerializer};
use bevy_reflect::{FromType, GetTypeRegistration, Reflect, ReflectDeserialize, TypeRegistry};
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ComponentSerializationFunction =
    fn(entity: &mut EntityMut, json: Value) -> Result<(), serde_json::Error>;

pub type NetworkChangeDetectionQuery = fn(world: &mut World) -> Vec<(EntityId, ComponentUpdate)>;
pub struct Registry {
    block_types: HashMap<block_type::BlockTypeId, block_type::BlockType>,
    network_change_detectors: HashMap<ComponentTypeId, NetworkChangeDetectionQuery>,
    type_registry: TypeRegistry,
    de_ser_funcs: HashMap<ComponentTypeId, ComponentSerializationFunction>,
}

pub struct RegistryBuilder {
    registry: Registry,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        let mut result = Self {
            registry: Registry {
                block_types: HashMap::new(),
                type_registry: TypeRegistry::default(),
                de_ser_funcs: HashMap::new(),
                network_change_detectors: HashMap::new(),
            },
        };
        //add default components
        result = result.with_component::<position::Position>();
        result = result.with_component::<player::Player>();
        result = result.with_component::<entity_id::EntityId>();
        result
    }
    pub fn with_component_and_callback<
        T: 'static + DeserializeOwned + Component + GetTypeRegistration + Reflect + Default,
    >(
        mut self,
        func: ComponentSerializationFunction,
    ) -> Self {
        self.registry.type_registry.register::<T>();
        let registration = self
            .registry
            .type_registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();

        registration.insert(<ReflectComponent as FromType<T>>::from_type());

        self.registry
            .de_ser_funcs
            .insert(ComponentTypeId::new::<T>(), func);
        self
    }
    pub fn with_component<
        T: 'static
            + DeserializeOwned
            + Component
            + GetTypeRegistration
            + Reflect
            + Default
            + Serialize,
    >(
        mut self,
    ) -> Self {
        self.register_reflect_component::<T>();

        self.add_deserialization_function::<T>();

        self.add_network_update_function::<T>();

        self
    }

    fn add_network_update_function<
        T: 'static
            + DeserializeOwned
            + Component
            + GetTypeRegistration
            + Reflect
            + Default
            + Serialize,
    >(
        &mut self,
    ) {
        self.registry
            .network_change_detectors
            .insert(get_type_id::<T>(), |w| {
                let mut res = Vec::new();
                let mut query = w.query::<(&T, ChangeTrackers<T>, Entity)>().for_each(
                    w,
                    |(comp, changed, entity)| {
                        if let Some(uid) = w
                            .get_resource::<UuidMap>()
                            .expect("UuidMap not in world")
                            .get_by_entity(entity)
                        {
                            let val = serde_json::to_value(comp)
                                .expect("Could not serialize in network function");
                            if changed.is_added() {
                                res.push((
                                    uid,
                                    ComponentUpdate::new(
                                        uid,
                                        get_type_id::<T>(),
                                        ComponentUpdateType::Added { packet: val },
                                    ),
                                ));
                            } else if changed.is_changed() {
                                res.push((
                                    uid,
                                    ComponentUpdate::new(
                                        uid,
                                        get_type_id::<T>(),
                                        ComponentUpdateType::Changed { packet: val },
                                    ),
                                ));
                            }
                        }
                    },
                );
                let map = w.get_resource::<UuidMap>().expect("UuidMap not in world");
                w.removed::<T>().for_each(|e| {
                    res.push((
                        map.get_by_entity(e)
                            .expect("Entity removed but not in map, see TODO in UUidMap"),
                        ComponentUpdate::new(
                            map.get_by_entity(e).unwrap(),
                            get_type_id::<T>(),
                            ComponentUpdateType::Removed,
                        ),
                    ));
                });
                res
            });
    }

    fn register_reflect_component<
        T: 'static + DeserializeOwned + Component + GetTypeRegistration + Reflect + Default,
    >(
        &mut self,
    ) {
        //due to shitty docs, I didn't know you also needed to register ReflectComponent.
        self.registry.type_registry.register::<T>();
        let registration = self
            .registry
            .type_registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();
        registration.insert(<ReflectComponent as FromType<T>>::from_type());
    }

    fn add_deserialization_function<
        T: 'static + DeserializeOwned + Component + GetTypeRegistration + Reflect + Default,
    >(
        &mut self,
    ) {
        self.registry
            .de_ser_funcs
            .insert(ComponentTypeId::new::<T>(), |mut entity, json| {
                let v: T = serde_json::from_value(json)?;
                entity.insert(v);
                Ok(())
            });
    }
    pub fn load_block_raws(mut self, path: &[&str], raws: &RawTree) -> RegistryBuilder {
        for block_raws in raws.search_for_all(path) {
            if let Some(block) = block_raws.get::<BlockType>() {
                self.registry.block_types.insert(block.get_id(), block);
            }
        }
        self
    }

    pub fn build(self) -> Registry {
        self.registry
    }
}

impl Registry {
    pub fn get_block_type(&self, canonical_name: &str) -> Option<&block_type::BlockType> {
        self.block_types.get(&hashing::string_hash(canonical_name))
    }
    pub fn type_registry(&self) -> &TypeRegistry {
        &self.type_registry
    }
    pub fn get_network_change_serialization(
        &self,
        w: &mut World,
    ) -> HashMap<EntityId, Vec<ComponentUpdate>> {
        //Collect all of the component updates in the world from the network change detectors
        let mut res: HashMap<EntityId, Vec<ComponentUpdate>> = HashMap::new();
        self.network_change_detectors.iter().for_each(|(comp, f)| {
            let r = f(w);
            for (eid, update) in r {
                match res.entry(eid) {
                    std::collections::hash_map::Entry::Occupied(mut v) => {
                        v.get_mut().push(update);
                    }
                    std::collections::hash_map::Entry::Vacant(v) => {
                        v.insert(vec![update]);
                    }
                }
            }
        });
        res
    }
    pub fn add_component_to_entity(
        &self,
        entity: &mut EntityMut,
        type_string: String,
        json: Value,
    ) -> () {
        match self.type_registry.get_with_name(&type_string) {
            Some(component_deserializer) => {
                match self
                    .de_ser_funcs
                    .get(&get_type_id_from_str(component_deserializer.name()))
                {
                    Some(ser_func) => {
                        ser_func(entity, json);
                    }
                    None => {
                        panic!(
                            "No serialization function for component {}",
                            component_deserializer.name()
                        )
                    }
                }
            }
            None => {
                panic!("No component type with id");
            }
        }
    }
}

#[test]
fn test_registry() {
    let rt = RawTree::new("./raws");
    let b = RegistryBuilder::new()
        .load_block_raws(&["block"], &rt)
        .build();
}
