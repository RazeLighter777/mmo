use std::sync::Arc;
use std::sync::Mutex;
use crate::world;
use crate::entity;
use crate::component;
use crate::game_event;
pub trait Generator: Send + Sync {
    fn update(&mut self) -> ();
    fn generate(&self, world : Arc<Mutex<&world::World>>,  ents : &Vec<entity::EntityId>) -> Vec<Box<dyn game_event::GameEventInterface>>;
    fn request(&self) -> Vec<component::ComponentTypeId>;
}