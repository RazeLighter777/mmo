use std::sync::Arc;
use std::sync::Mutex;

use crate::component;
use crate::entity;
use crate::game_event;
use crate::generator;
use crate::mass;
use crate::pos;
use crate::world;
use std::net::{TcpListener, TcpStream};

pub struct Positioner {}

impl generator::Generator for Positioner {
    fn update(&mut self) {}
    fn generate(
        &self,
        world: Arc<&world::World>,
        ents: &Vec<entity::EntityId>,
    ) -> Vec<Box<dyn game_event::GameEventInterface>> {
        for e in ents {
            let w = world.clone();
            println!(
                "Positioner : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<pos::Pos>()
                    .unwrap()
                    .dat()
                    .x
            );
        }
        Vec::new()
    }
    fn request(&self) -> Vec<component::ComponentTypeId> {
        vec![component::get_type_id::<pos::Pos>()]
    }
}
