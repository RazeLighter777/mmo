use bevy_ecs::prelude::Entity;

use crate::server_request_type::Direction;

pub struct MovementEvent {
    direction: Direction,
    entity: Entity,
}
