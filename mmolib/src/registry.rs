use std::sync::Arc;
use std::{collections::HashMap, fmt};

use crate::component::ComponentTypeId;
use crate::game_world::GameWorld;
use crate::hashing;
use crate::raws::Raw;
use crate::{
    block_type,
    component::{self, ComponentDataType},
    entity,
    raws::RawTree,
};
use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_ecs::world::EntityMut;
use bevy_reflect::serde::{ReflectSerializer, ReflectDeserializer};
use bevy_reflect::{TypeRegistry, GetTypeRegistration, ReflectDeserialize};
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::Deserialize;
use serde_json::Value;

pub type ComponentSerializationFunction =
    fn(entity: &mut EntityMut, json: Value) -> Result<(), serde_json::Error>;

pub struct Registry {
    block_types: HashMap<block_type::BlockTypeId, block_type::BlockType>,
    type_registry: TypeRegistry,
    ser_funcs : HashMap<String, ComponentSerializationFunction>
}

pub struct RegistryBuilder {
    registry: Registry,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: Registry {
                block_types: HashMap::new(),
                type_registry: TypeRegistry::default(),
                ser_funcs : HashMap::new()
            },
        }
    }
    pub fn with_component<T: ComponentDataType + 'static + DeserializeOwned + Component + GetTypeRegistration>(
        mut self,
    ) -> Self {
        self.registry.type_registry.register::<T>();
        self.registry.ser_funcs.insert(T::get_type_registration().name().to_owned(), |mut entity, json| {
            let v : T = serde_json::from_value(json)?;
            entity.insert(v);
            Ok(())
        } );
        self
    }
    pub fn load_block_raws(mut self, path: &[String], raws: &RawTree) -> RegistryBuilder {
        for block_raws in raws.search_for_all(path) {
            match serde_json::from_value(block_raws.dat().clone()) {
                Ok(v) => {
                    let b: block_type::BlockType = v;
                    self.registry
                        .block_types
                        .insert(hashing::string_hash(b.get_canonical_name()), b);
                }
                Err(e) => {
                    println!("Error deserializing block type {}", e)
                }
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
    pub fn add_component_to_entity(
        &self,
        entity: &mut EntityMut,
        type_string : String,
        json: Value,
    ) -> () {
        match self.type_registry.get_with_name(&type_string) {
            Some(component_deserializer) => {
                match self.ser_funcs.get(component_deserializer.name()) {
                    Some(ser_func) => {
                        ser_func(entity, json);
                    }
                    None => {
                        println!("No serialization function for component {}", component_deserializer.name())
                    }
                }
            }
            None => {
                println!("No component type with id");
            }
        }
    }
}

#[test]
fn test_registry() {
    let rt = RawTree::new("./raws");
    let b = RegistryBuilder::new()
        .load_block_raws(&["block".to_owned()], &rt)
        .build();
}
