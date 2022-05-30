use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::block_type;
use crate::entity;



const CHUNK_SIZE : usize = 32;

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    blocks : [[block_type::BlockTypeId; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip_serializing)]
    entity_position_cache : HashSet<entity::EntityId>
}

impl Chunk {
    pub fn new(dat : &[u8]) -> Result<Chunk,serde_cbor::Error> {
        let res = serde_cbor::from_slice(dat)?;
        Ok(res)
    }
    pub fn contains(&self, entity : entity::EntityId) -> bool {
        self.entity_position_cache.contains(&entity)
    }
    pub fn add(&mut self, entity : entity::EntityId) {
        self.entity_position_cache.insert(entity);
    }
    pub fn remove(&mut self, entity : entity::EntityId) -> bool {
        self.entity_position_cache.remove(&entity)
    }

}

pub type ChunkId = u64;

pub type Position = (u32, u32);

pub fn chunk_id_from_position(position : Position) -> ChunkId {
    (u64::from(position.0 / CHUNK_SIZE as u32) << 32) | u64::from(position.1 / CHUNK_SIZE as u32)
}
pub fn convert_to_chunk_relative_position(position : Position) -> Position {
    (position.0 & (CHUNK_SIZE as u32 - 1), position.1 & (CHUNK_SIZE as u32 - 1))
}
pub fn position_of_chunk(chunk_id : ChunkId) -> Position {
    ((chunk_id >> 32).try_into().unwrap(), chunk_id as u32)
}

#[test]
fn test_chunks() {
    let p : Position = (32,64);
    assert_eq!(position_of_chunk(chunk_id_from_position(p)), (1,2));
    assert_eq!(convert_to_chunk_relative_position(p), (0,0))
}