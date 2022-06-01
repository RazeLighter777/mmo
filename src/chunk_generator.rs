use crate::{chunk::{self, Chunk, Position}, raws::RawTree, world};





pub trait ChunkGenerator {
    fn generate_chunk(&self, chunk_id : chunk::ChunkId) -> Chunk;
    fn query_attributes(&self, position : Position) -> chunk::LocationAttributes;
}