use std::sync::Arc;
use std::sync::Mutex;
use mmolib;
use mmolib::generator;
use mmolib::world;
pub struct Gravity {}

impl generator::Generator for Gravity {
    fn update(&mut self) {}
    fn generate(
        &self,
        world: Arc<&world::World>,
        ents: &Vec<mmolib::entity::EntityId>,
    ) -> Vec<Box<dyn mmolib::game_event::GameEventInterface>> {
        for e in ents {
            let w = world.clone();
            println!(
                "gravity : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<mmolib::pos::Pos>()
                    .unwrap()
                    .dat()
                    .x
            );
            println!(
                "gravity : {}",
                w.get_entity_by_id(*e)
                    .unwrap()
                    .get::<mmolib::mass::Mass>()
                    .unwrap()
                    .dat()
                    .m
            );
        }
        Vec::new()
    }
    fn request(&self) -> Vec<mmolib::component::ComponentTypeId> {
        vec![
            mmolib::component::get_type_id::<mmolib::pos::Pos>(),
            mmolib::component::get_type_id::<mmolib::mass::Mass>(),
        ]
    }
}
