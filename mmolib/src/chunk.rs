use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::block_type;
use crate::entity;

pub const CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    blocks: [[block_type::BlockTypeId; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip_serializing)]
    entity_position_cache: HashSet<entity::EntityId>,
}

pub struct LocationAttributes {
    temperature: f32,
    altitude: f32,
    humidity: f32,
}

impl Chunk {
    pub fn new(dat: &[u8]) -> Result<Chunk, serde_cbor::Error> {
        let res = serde_cbor::from_slice(dat)?;
        Ok(res)
    }
    pub fn new_from_array(blocks: [[block_type::BlockTypeId; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
        Self {
            blocks: blocks,
            entity_position_cache: HashSet::new(),
        }
    }
    pub fn contains(&self, entity: entity::EntityId) -> bool {
        self.entity_position_cache.contains(&entity)
    }
    pub fn add(&mut self, entity: entity::EntityId) {
        self.entity_position_cache.insert(entity);
    }
    pub fn remove(&mut self, entity: entity::EntityId) -> bool {
        self.entity_position_cache.remove(&entity)
    }
    pub fn get_entities(&self) -> Vec<entity::EntityId> {
        self.entity_position_cache.iter().cloned().collect()
    }
}

pub type ChunkId = u64;

pub type Position = (u32, u32);

pub fn chunk_id_from_position(position: Position) -> ChunkId {
    (u64::from(position.0 / CHUNK_SIZE as u32) << 32) | u64::from(position.1 / CHUNK_SIZE as u32)
}
pub fn convert_to_chunk_relative_position(position: Position) -> Position {
    (
        position.0 & (CHUNK_SIZE as u32 - 1),
        position.1 & (CHUNK_SIZE as u32 - 1),
    )
}
pub fn position_of_chunk(chunk_id: ChunkId) -> Position {
    ((chunk_id >> 32).try_into().unwrap(), chunk_id as u32)
}

pub fn distance_between_position(a: Position, b: Position) -> f32 {
    let (x1, y1) = a;
    let (x2, y2) = b;
    ((x1 - x2) as f32).hypot((y1 - y2) as f32)
}

#[test]
fn test_chunks() {
    let p: Position = (32, 64);
    assert_eq!(position_of_chunk(chunk_id_from_position(p)), (1, 2));
    assert_eq!(convert_to_chunk_relative_position(p), (0, 0))
}
