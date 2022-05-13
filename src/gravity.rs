use std::sync::Arc;
use std::sync::Mutex;

use crate::entity;
use crate::generator;
use crate::world;
use crate::component;
use crate::pos;
use crate::mass;
pub struct Gravity {

}

impl generator::Generator for Gravity {
    
    fn generate(&self, world : Arc<Mutex<&world::World>>,  ents : &Vec<entity::EntityId>) {
        for e in ents {        
            let w = world.lock();
            println!("{}",w.unwrap().get_entity_by_id(*e).unwrap().get::<pos::Pos>().unwrap().dat().x);
        }
    }
    fn request(&self) -> Vec<component::ComponentTypeId> {
        vec![component::get_type_id::<pos::Pos>(), component::get_type_id::<mass::Mass>()]
    }
}