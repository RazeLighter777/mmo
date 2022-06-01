use std::{collections::HashMap, fmt};


use serde_json::Value;

use crate::{block_type, component::{self, ComponentInterface, ComponentDataType, Component}, entity, pos, raws::RawTree};

pub type ComponentSerializationFunction = fn(dat : Value, parent : entity::EntityId) -> Box<dyn ComponentInterface>;
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
        self.registry.component_types.insert(component::get_type_id::<T>(), |dat : Value, parent : entity::EntityId| {
            let x : T = serde_json::from_value(dat).expect(&format!("Could not deserializae component of type {:?}", std::any::type_name::<T>()));
            let comp = Component::<T>::new(x,parent);
            let component_box : Box<dyn ComponentInterface> = Box::new(comp);
            component_box
        });
        self
    }
    pub fn load_block_raws(self, path : &Vec<String>, raws : RawTree) -> RegistryBuilder {
        
        todo!()
    }
    pub fn build(self) -> Registry {
        self.registry
    }
}


impl Registry {
    
}

#[test]
fn test_registry()  {
    let b = RegistryBuilder::new().with_component::<pos::Pos>().build();
}