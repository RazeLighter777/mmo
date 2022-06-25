use bevy_ecs::prelude::Component;
use bevy_reflect::Reflect;
use bevy_reflect::ReflectDeserialize;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Default, Serialize, Deserialize, Clone, PartialEq, Debug, Component)]
#[reflect_value(Serialize, PartialEq, Deserialize)]
pub struct Player {
    pub username: String,
    pub last_ping_timestamp: u64,
}

impl Player {
    pub fn update_timestamp(&mut self) {
        self.last_ping_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}
