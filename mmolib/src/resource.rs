use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ResourceId {
    StoneFloor,
    Grass1,
    Dirt1,
    Acid1,
    Acid2,
    
}

#[derive(Clone)]
pub enum ResourceType<'a> {
    StaticImage(&'static str),
    Animation(&'a [&'static str]),
    Sound(&'static str, f32),
}
pub fn spawn_resource_map() -> HashMap<ResourceId,ResourceType<'static>> {
    [
        (ResourceId::StoneFloor, ResourceType::StaticImage("images/sprite/StoneFloor.png")),
        (ResourceId::BasicStone1, ResourceType::StaticImage("images/sprite/BasicStone1.png")),
        (ResourceId::BasicWater1, ResourceType::StaticImage("images/sprite/BasicWater1.png")),
        (ResourceId::BasicWater2, ResourceType::StaticImage("images/sprite/BasicWater2.png")),
        (ResourceId::Sand1, ResourceType::StaticImage("images/sprite/Sand1.png")),
        (ResourceId::Grass1, ResourceType::StaticImage("images/sprite/Grass1.png")),
        
    ]
    .iter()
    .cloned()
    .collect()
}
