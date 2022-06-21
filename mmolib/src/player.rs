use bevy_ecs::prelude::Component;
use serde::Deserialize;

#[derive(Component, Deserialize)]
pub struct Player {
    username: String,
    last_ping_timestamp: u64,
}

impl Player {
    pub fn update_timestamp(&mut self) {
        self.last_ping_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}
