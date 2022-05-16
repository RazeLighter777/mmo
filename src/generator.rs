use crate::component;
use crate::entity;
use crate::game_event;
use crate::world;
use std::sync::Arc;
use std::sync::Mutex;
pub trait Generator: Sync + Send {
    fn update(&mut self) -> ();
    fn generate(
        &self,
        world: Arc<Mutex<&world::World>>,
        ents: &Vec<entity::EntityId>,
    ) -> Vec<Box<dyn game_event::GameEventInterface>>;
    fn request(&self) -> Vec<component::ComponentTypeId>;
}
