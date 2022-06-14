use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Clone, Hash, PartialEq, Eq,Debug, Serialize, Deserialize)]
pub enum ResourceId {
    StoneFloor,
    BasicStone1,
    BasicWater1,
    BasicWater2,
    Sand1,
    Grass1,
}

pub fn spawn_resource_map() -> HashMap<ResourceId, &'static str> {
    [
        (ResourceId::StoneFloor, "images/sprite/StoneFloor.png"),
        (ResourceId::BasicStone1, "images/sprites/BasicStone1.png"),
        (ResourceId::BasicWater1, "images/sprites/BasicWater1.png"),
        (ResourceId::BasicWater2, "images/sprites/BasicWater2.png"),
        (ResourceId::Sand1, "images/sprites/Sand1.png.png"),
        (ResourceId::Grass1, "images/sprites/Grass1.png"),
    ]
    .iter()
    .cloned()
    .collect()
}
