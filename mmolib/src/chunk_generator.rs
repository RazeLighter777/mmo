use crate::{chunk, world};
pub trait ChunkGenerator: Send + Sync {
    fn generate_chunk(&self, chunk_id: chunk::ChunkId, world: &world::World) -> chunk::Chunk;
    fn query_attributes(&self, position: chunk::Position) -> chunk::LocationAttributes;
}
