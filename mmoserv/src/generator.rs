use crate::world;
use std::sync::Arc;
use std::sync::Mutex;
use mmolib;
pub trait Generator: Sync + Send {
    fn update(&mut self) -> ();
    fn generate(
        &self,
        world: Arc<&world::World>,
        ents: &Vec<mmolib::entity::EntityId>,
    ) -> Vec<Box<dyn mmolib::game_event::GameEventInterface>>;
    fn request(&self) -> Vec<mmolib::component::ComponentTypeId>;
}
