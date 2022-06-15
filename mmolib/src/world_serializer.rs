use crate::{chunk, chunk_generator, component, entity, world};

pub trait WorldSerializer: Send + Sync {
    fn set_generator(&mut self, gen: Box<dyn chunk_generator::ChunkGenerator>);
    fn retrieve_chunk_and_entities(
        &mut self,
        chunk_id: chunk::ChunkId,
        world: &world::World,
    ) -> (chunk::Chunk, Vec<entity::Entity>);
    fn save_chunks(
        &mut self,
        chunk: Vec<(chunk::ChunkId, &chunk::Chunk)>,
        world: &world::World,
        loaded: bool,
    );
    fn delete_components(&mut self, components: Vec<component::ComponentId>);
    fn delete_entities(&mut self, entities: Vec<entity::EntityId>);
    fn save_entities(& self, entities: Vec<&entity::Entity>, world: &world::World);
}
