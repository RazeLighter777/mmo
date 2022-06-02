use std::sync::Arc;
use std::sync::Mutex;

use crate::component;
use crate::entity;
use crate::game_event;
use crate::generator;
use crate::mass;
use crate::pos;
use crate::world;
pub struct Gravity {}

impl generator::Generator for Gravity {
    fn update(&mut self) {}
    fn generate(
        &self,
        world: Arc<&world::World>,
        ents: &Vec<entity::EntityId>,
    ) -> Vec<Box<dyn game_event::GameEventInterface>> {
        for e in ents {
            let w = world.clone();
            println!(
                "gravity : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<pos::Pos>()
                    .unwrap()
                    .dat()
                    .x
            );
            println!(
                "gravity : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<mass::Mass>()
                    .unwrap()
                    .dat()
                    .m
            );
        }
        Vec::new()
    }
    fn request(&self) -> Vec<component::ComponentTypeId> {
        vec![
            component::get_type_id::<pos::Pos>(),
            component::get_type_id::<mass::Mass>(),
        ]
    }
}
