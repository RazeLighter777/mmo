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

pub fn spawn_resource_map() -> HashMap<ResourceId, &'static str> {
    [
        (ResourceId::StoneFloor, "images/sprite/StoneFloor.png"),
        (ResourceId::Grass1, "images/sprites/Grass1.png"),
        (ResourceId::Dirt1, "images/sprites/Dirt1.png"),
        (ResourceId::Acid1, "images/sprites/Acid1.png"),
        (ResourceId::Acid2, "images/sprites/Acid2.png"),
        
        
        

    ].iter().cloned().collect()
}
