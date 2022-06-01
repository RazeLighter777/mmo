use std::sync::Arc;
use std::{collections::HashMap, fmt};


use serde_json::Value;
use crate::{hashing, context};
use crate::raws::Raw;
use crate::{block_type, component::{self, ComponentInterface, ComponentDataType, Component}, entity, pos, raws::RawTree};

pub type ComponentSerializationFunction = fn(dat : Value, parent : entity::EntityId, context : Arc<context::Context>) -> Box<dyn ComponentInterface>;
pub struct Registry {
    block_types : HashMap<block_type::BlockTypeId, block_type::BlockType>,
    component_types : HashMap<component::ComponentTypeId, ComponentSerializationFunction>,
}

pub struct RegistryBuilder {
    registry : Registry
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self { registry: Registry { block_types: HashMap::new(), component_types: HashMap::new() } }
    }
    pub fn with_component<T: ComponentDataType + 'static>(mut self) -> Self {
        self.registry.component_types.insert(component::get_type_id::<T>(), |dat : Value, parent : entity::EntityId, context : Arc<context::Context>| {
            let x : T = serde_json::from_value(dat).expect(&format!("Could not deserializae component of type {:?}", std::any::type_name::<T>()));
            let comp = Component::<T>::new(x,parent, context);
            let component_box : Box<dyn ComponentInterface> = Box::new(comp);
            component_box
        });
        self
    }
    pub fn load_block_raws(mut self, path : &[String], raws : &RawTree) -> RegistryBuilder {
        for block_raws in raws.search_for_all(path) {
            match serde_json::from_value(block_raws.dat().clone())  {
                Ok(v) => {
                    let b : block_type::BlockType = v;
                    self.registry.block_types.insert(hashing::string_hash(b.get_canonical_name()) , b);
                },
                Err(e) => {
                    println!("Error deserializing block type {}", e)
                },
            }
        }
        self
    }
    pub fn build(self) -> Registry {
        self.registry
    }
}


impl Registry {
    pub fn get_block_type(&self, canonical_name : &str) -> Option<&block_type::BlockType> {
        self.block_types.get(&hashing::string_hash(canonical_name))
    }
    pub fn generate_component(&self, dat : Value, entity_id : entity::EntityId, type_id : u64, context : Arc<context::Context>) -> Option<Box<dyn ComponentInterface>> {
        match self.component_types.get(&type_id) {
            Some(gen) => {
                Some(gen(dat, entity_id, context))
            },
            None => {
                None
            },
        }
    }
}

#[test]
fn test_registry()  {
    let rt = RawTree::new("./raws");
    let b = RegistryBuilder::new().with_component::<pos::Pos>().load_block_raws(&["block".to_owned()], &rt).build();
}