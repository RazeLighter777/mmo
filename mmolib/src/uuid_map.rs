use std::collections::HashMap;

use bevy_ecs::entity::Entity;

use crate::entity_id::EntityId;

pub struct UuidMap {
    uuid_to_entity: HashMap<EntityId, Entity>,
    entity_to_uuid: HashMap<Entity, EntityId>,
}

impl UuidMap {
    pub fn new() -> Self {
        Self {
            uuid_to_entity: HashMap::new(),
            entity_to_uuid: HashMap::new(),
        }
    }
    pub fn add(&mut self, uuid: EntityId, entity: Entity) {
        self.uuid_to_entity.insert(uuid, entity);
        self.entity_to_uuid.insert(entity, uuid);
    }

    pub fn get_by_entity(&self, entity: Entity) -> Option<EntityId> {
        self.entity_to_uuid.get(&entity).cloned()
    }
    pub fn get(&self, uuid: EntityId) -> Option<&Entity> {
        self.uuid_to_entity.get(&uuid)
    }
    pub(crate) fn remove(&mut self, entity: Entity) {
        match self.entity_to_uuid.get(&entity) {
            Some(uuid) => {
                self.uuid_to_entity.remove(uuid);
                self.entity_to_uuid.remove(&entity);
            }
            None => {}
        }
    }
}
