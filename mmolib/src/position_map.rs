use std::collections::HashMap;

use bevy_ecs::{entity::Entity, prelude::Changed};

use crate::{chunk, position};

use bevy_ecs::prelude::*;

pub struct PositionMap {
    entities: HashMap<Entity, chunk::Position>,
    chunk_mapping: HashMap<chunk::ChunkId, Vec<Entity>>,
}

impl PositionMap {
    pub fn new() -> Self {
        PositionMap {
            entities: HashMap::new(),
            chunk_mapping: HashMap::new(),
        }
    }
    pub fn get_position(&self, entity: Entity) -> Option<chunk::Position> {
        self.entities.get(&entity).cloned()
    }
    pub fn get_entities_in_chunk(&self, chunk_id: chunk::ChunkId) -> Option<&Vec<Entity>> {
        self.chunk_mapping.get(&chunk_id)
    }
    pub(crate) fn add(&mut self, entity: Entity, position: chunk::Position) {
        self.entities.insert(entity, position);
        let chunk_id = chunk::chunk_id_from_position(position);
        match self.chunk_mapping.entry(chunk_id) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().push(entity);
            }
            std::collections::hash_map::Entry::Vacant(mut e) => {
                e.insert(vec![entity]);
            }
        }
    }
    pub(crate) fn remove(&mut self, entity: Entity) {
        match self.entities.remove(&entity) {
            Some(position) => {
                let chunk_id = chunk::chunk_id_from_position(position);
                let mut chunk_entities = self.chunk_mapping.get_mut(&chunk_id).unwrap();
                chunk_entities.retain(|&x| x != entity);
            }
            None => {}
        }
    }
}

pub fn update_position_map_on_position_change(
    mut position_map: ResMut<PositionMap>,
    mut commands: Commands,
    mut query: Query<(Entity, &position::Position), Changed<position::Position>>,
) {
    for (entity, pos) in query.iter() {
        position_map.remove(entity);
        position_map.add(entity, pos.pos);
    }
}

pub fn update_position_map_on_position_removal(
    mut position_map: ResMut<PositionMap>,
    mut commands: Commands,
    removed_positions: RemovedComponents<position::Position>,
) {
    for entity in removed_positions.iter() {
        position_map.remove(entity);
    }
}
pub fn update_position_test_method(mut query: Query<(Entity, &mut position::Position)>) {
    for (entity, mut position) in query.iter_mut() {
        position.pos = (position.pos.0 + 1, position.pos.1 + 1);
    }
}
