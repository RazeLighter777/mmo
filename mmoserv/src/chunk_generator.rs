use mmolib;
use mmolib::world;

pub trait ChunkGenerator {
    fn generate_chunk(&self, chunk_id: mmolib::chunk::ChunkId) -> mmolib::chunk::Chunk;
    fn query_attributes(
        &self,
        position: mmolib::chunk::Position,
    ) -> mmolib::chunk::LocationAttributes;
}
