use std::collections::{HashMap, HashSet};

use crate::chunk::{Chunk, ChunkId, Position};

pub struct ChunkMap {
    chunks: HashMap<ChunkId, Chunk>,
    change_tracking: HashSet<ChunkId>,
}

impl ChunkMap {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            change_tracking: HashSet::new(),
        }
    }
    pub fn add(&mut self, chunk_id: ChunkId, chunk: Chunk) {
        self.chunks.insert(chunk_id, chunk);
        self.change_tracking.insert(chunk_id);
    }
    pub fn is_chunk_changed(&self, chunk_id: ChunkId) -> bool {
        self.change_tracking.contains(&chunk_id)
    }
    pub fn get(&self, chunk_id: ChunkId) -> Option<&Chunk> {
        self.chunks.get(&chunk_id)
    }
    pub fn remove(&mut self, chunk_id: ChunkId) -> Option<Chunk> {
        self.chunks.remove(&chunk_id)
    }
    pub fn contains(&self, chunk_id: ChunkId) -> bool {
        self.chunks.contains_key(&chunk_id)
    }
    pub fn get_loaded_chunks(&self) -> Vec<&ChunkId> {
        self.chunks.keys().collect()
    }
    pub fn clear_trackers(&mut self) {
        self.change_tracking.clear();
    }
}
