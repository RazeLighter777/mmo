use crate::{chunk, entity, world};

pub trait WorldSerializer {
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
    fn delete_entities(&mut self, entities: Vec<&entity::Entity>, world: &world::World);
    fn save_entities(&mut self, entities: Vec<&entity::Entity>, world: &world::World);
}
