pub struct FlatWorldGenerator {

}


impl FlatWorldGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl mmolib::chunk_generator::ChunkGenerator for FlatWorldGenerator {
    fn generate_chunk(&self, chunk_id: mmolib::chunk::ChunkId, world : &mmolib::world::World) -> mmolib::chunk::Chunk {
        
    }

    fn query_attributes(&self, position: mmolib::chunk::Position) -> mmolib::chunk::LocationAttributes {
        todo!()
    }
}