use bevy_ecs::prelude::Entity;

use crate::entity_id::EntityId;

pub struct EntityDeletionList {
    entities: Vec<EntityId>,
}

impl EntityDeletionList {
    pub(crate) fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
    pub fn add(&mut self, entity: EntityId) {
        self.entities.push(entity);
    }
    pub(crate) fn get_entities(&mut self) -> Vec<EntityId> {
        let res = self.entities.clone();
        self.entities.clear();
        res
    }
}
