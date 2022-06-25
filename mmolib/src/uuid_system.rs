use bevy_ecs::{prelude::*, world::EntityMut};

use crate::{chunk::Position, entity_id::EntityId, uuid_map};

pub fn uuid_system(
    mut uuid_map: ResMut<uuid_map::UuidMap>,
    query: Query<(Entity), Without<EntityId>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        let uuid = EntityId::new();
        //insert the entity into the commands
        let mut mut_entity = commands.get_or_spawn(entity);
        uuid_map.add(uuid, entity);
        mut_entity.insert(uuid);
    }
}
pub fn on_remove_uuid(
    mut uuid_map: ResMut<uuid_map::UuidMap>,
    iids_gone: RemovedComponents<EntityId>,
) {
    for x in iids_gone.iter() {
        uuid_map.remove(x);
    }
}
