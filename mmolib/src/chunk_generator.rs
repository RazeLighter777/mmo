use std::fmt::Debug;

use crate::{chunk, game_world, registry::Registry};
pub trait ChunkGenerator: Send + Sync + Debug {
    fn generate_chunk(
        &self,
        chunk_id: chunk::ChunkId,
        world: &game_world::GameWorld,
        registry : &Registry,
    ) -> chunk::Chunk;
    fn query_attributes(&self, position: chunk::Position) -> chunk::LocationAttributes;
}
