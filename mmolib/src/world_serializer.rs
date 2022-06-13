use crate::{chunk, entity};

pub trait WorldSerializer {
    fn retrieve_chunk_and_entities(chunk_id: chunk::ChunkId)
        -> (chunk::Chunk, Vec<entity::Entity>);
    fn save_chunk(chunk: chunk::Chunk);
    fn save_entities(entities: Vec<entity::Entity>);
}
