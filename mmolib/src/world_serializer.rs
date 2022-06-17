use std::fmt::Debug;

use async_trait::async_trait;

use crate::{chunk, chunk_generator, component, entity, world};


#[async_trait]
pub trait WorldSerializer: Send + Sync + Debug {
    //this function sets the generator for the world serializer
    async fn set_generator(&mut self, gen: Box<dyn chunk_generator::ChunkGenerator>);
    /**
     * This function loads a chunk and its entities from the database.
     * If the chunk is not found in the database, it will be generated and saved to the database.
     */
    async fn retrieve_chunk_and_entities(
        &self,
        chunk_id: chunk::ChunkId,
        world: &world::World,
    ) -> (
        chunk::Chunk,
        Vec<entity::Entity>,
        Vec<Box<dyn component::ComponentInterface>>,
    );
    async fn save_chunks(
        &self,
        chunk: Vec<(chunk::ChunkId, &chunk::Chunk)>,
        world: &world::World,
        loaded: bool,
    );
    async fn delete_components(&mut self, components: Vec<component::ComponentId>);
    async fn delete_entities(&mut self, entities: Vec<entity::EntityId>);
    async fn save_entities(&self, entities: Vec<&entity::Entity>, world: &world::World);
}
