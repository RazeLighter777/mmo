use std::sync::Arc;
use std::sync::Mutex;

use mmolib::generator;
use mmolib::world;
use mmolib;
use std::net::{TcpListener, TcpStream};

pub struct Positioner {}

impl generator::Generator for Positioner {
    fn update(&mut self) {}
    fn generate(
        &self,
        world: Arc<&world::World>,
        ents: &Vec<mmolib::entity::EntityId>,
    ) -> Vec<Box<dyn mmolib::game_event::GameEventInterface>> {
        for e in ents {
            let w = world.clone();
            println!(
                "Positioner : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<mmolib::pos::Pos>()
                    .unwrap()
                    .dat()
                    .x
            );
        }
        Vec::new()
    }
    fn request(&self) -> Vec<mmolib::component::ComponentTypeId> {
        vec![mmolib::component::get_type_id::<mmolib::pos::Pos>()]
    }
}
