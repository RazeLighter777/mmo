use std::fmt::Debug;

use crate::{chunk, game_world};
pub trait ChunkGenerator: Send + Sync + Debug {
    fn generate_chunk(
        &self,
        chunk_id: chunk::ChunkId,
        world: &game_world::GameWorld,
    ) -> chunk::Chunk;
    fn query_attributes(&self, position: chunk::Position) -> chunk::LocationAttributes;
}
