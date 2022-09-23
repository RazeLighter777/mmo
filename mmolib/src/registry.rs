
use std::sync::Arc;
use std::{collections::HashMap, fmt};

use crate::block_type::BlockType;
use crate::component::{ComponentTypeId, Networked};
use crate::game_world::GameWorld;
use crate::raws::Raw;
use crate::util::GetStatic;
use crate::{
    block_type,
    component::{self, ComponentDataType},
    entity_id,
    raws::RawTree,
};
use crate::{hashing, player, position};
use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_ecs::world::EntityMut;
use bevy_reflect::serde::{ReflectDeserializer, ReflectSerializer};
use bevy_reflect::{FromType, GetTypeRegistration, Reflect, ReflectDeserialize, TypeRegistry};
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::Deserialize;
use serde_json::Value;

pub type ComponentSerializationFunction =
    fn(entity: &mut EntityMut, json: Value) -> Result<(), serde_json::Error>;

pub struct Registry {
    block_types: HashMap<block_type::BlockTypeId, block_type::BlockType>,
    type_registry: TypeRegistry,
    ser_funcs: HashMap<String, ComponentSerializationFunction>,
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
                ser_funcs: HashMap::new(),
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
            .ser_funcs
            .insert(T::get_type_registration().name().to_owned(), func);
        self
    }
    pub fn with_component<
        T: 'static + DeserializeOwned + Component + GetTypeRegistration + Reflect + Default,
    >(
        mut self,
    ) -> Self {
        //due to shitty docs, I didn't know you also needed to register ReflectComponent.
        self.registry.type_registry.register::<T>();
        let registration = self
            .registry
            .type_registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();
        registration.insert(<ReflectComponent as FromType<T>>::from_type());

        self.registry.ser_funcs.insert(
            T::get_type_registration().name().to_owned(),
            |mut entity, json| {
                let v: T = serde_json::from_value(json)?;
                entity.insert(v);
                Ok(())
            },
        );
        //Check if component is networked
        
        self
    }
    pub fn load_block_raws(mut self, path: &[String], raws: &RawTree) -> RegistryBuilder {
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
    pub fn add_component_to_entity(
        &self,
        entity: &mut EntityMut,
        type_string: String,
        json: Value,
    ) -> () {
        match self.type_registry.get_with_name(&type_string) {
            Some(component_deserializer) => {
                match self.ser_funcs.get(component_deserializer.name()) {
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
        .load_block_raws(&["block".to_owned()], &rt)
        .build();
}
