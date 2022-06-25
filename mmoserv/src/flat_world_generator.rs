use mmolib::{block_type, chunk::CHUNK_SIZE};
#[derive(Debug)]
pub struct FlatWorldGenerator {}

impl FlatWorldGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl mmolib::chunk_generator::ChunkGenerator for FlatWorldGenerator {
    fn generate_chunk(
        &self,
        chunk_id: mmolib::chunk::ChunkId,
        registry: &mmolib::registry::Registry,
    ) -> mmolib::chunk::Chunk {
        let mut blocks: [[block_type::BlockTypeId; CHUNK_SIZE]; CHUNK_SIZE] = [[registry
            .get_block_type("stonefloor")
            .expect("could not find stone floor")
            .get_id();
            32]; 32];
        mmolib::chunk::Chunk::new_from_array(blocks)
    }

    fn query_attributes(
        &self,
        position: mmolib::chunk::Position,
    ) -> mmolib::chunk::LocationAttributes {
        todo!()
    }
}
