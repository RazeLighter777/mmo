use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ResourceId {
    StoneFloor,
    Grass1,
    Dirt1,
    AcidAnimation,
}

#[derive(Clone)]
pub enum ResourceType {
    StaticImage(&'static str),
    Animation(&'static [&'static str]),
    Sound(&'static str, f32),
    RotationalImage {
        left : &'static str,
        right : &'static str,
        front : &'static str,
        back : &'static str,
    }
}
pub fn spawn_resource_map() -> HashMap<ResourceId, ResourceType> {
    [
        (
            ResourceId::StoneFloor,
            ResourceType::StaticImage("images/sprite/StoneFloor.png"),
        ),
        (
            ResourceId::Grass1,
            ResourceType::StaticImage("images/sprite/Grass1.png"),
        ),
        (
            ResourceId::AcidAnimation,
            ResourceType::Animation(&["images/sprite/Acid1.png", "images/sprite/Acid2.png"]),
        ),
    ]
    .iter()
    .cloned()
    .collect()
}
