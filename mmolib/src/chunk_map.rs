use std::collections::HashMap;

use crate::chunk::{Chunk, ChunkId, Position};

pub struct ChunkMap {
    chunks: HashMap<ChunkId, Chunk>,
}

impl ChunkMap {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    pub fn add(&mut self, chunk_id: ChunkId, chunk: Chunk) {
        self.chunks.insert(chunk_id, chunk);
    }
    pub fn get(&self, chunk_id: ChunkId) -> Option<&Chunk> {
        self.chunks.get(&chunk_id)
    }
}
