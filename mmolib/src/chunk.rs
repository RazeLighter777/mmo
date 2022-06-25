use std::collections::HashSet;
use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::block_type;
use crate::entity_id;

pub const CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk {
    blocks: [[block_type::BlockTypeId; CHUNK_SIZE]; CHUNK_SIZE],
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
        Self { blocks: blocks }
    }
}
#[derive(Eq, Hash, PartialEq, Copy, Clone, Deserialize, Debug)]
pub struct ChunkId(u64);

impl ChunkId {
    pub fn new_raw(y: u64) -> Self {
        ChunkId(y)
    }
    pub fn id(&self) -> u64 {
        self.0
    }
}
pub type Position = (u32, u32);

pub fn chunk_id_from_position(position: (u32, u32)) -> ChunkId {
    ChunkId::new_raw(
        ChunkId::new_raw(u64::from(position.0 / CHUNK_SIZE as u32) << 32).id()
            | u64::from(position.1 / CHUNK_SIZE as u32),
    )
}
pub fn convert_to_chunk_relative_position(position: Position) -> Position {
    (
        position.0 & (CHUNK_SIZE as u32 - 1),
        position.1 & (CHUNK_SIZE as u32 - 1),
    )
}
pub fn position_of_chunk(chunk_id: ChunkId) -> Position {
    (
        (chunk_id.id() >> 32).try_into().unwrap(),
        chunk_id.id() as u32,
    )
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

impl Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = position_of_chunk(ChunkId(self.0));
        write!(f, "chunk<{},{}>", x, y)
    }
}
