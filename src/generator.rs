use std::sync::Arc;
use std::sync::Mutex;
use crate::world;
use crate::entity;
use crate::component;
pub trait Generator: Send + Sync {
    fn generate(&self, world : Arc<Mutex<&world::World>>,  ents : &Vec<entity::EntityId>);
    fn request(&self) -> Vec<component::ComponentTypeId>;
}