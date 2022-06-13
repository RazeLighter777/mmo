use std::collections::HashMap;

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum ResourceId {
    StoneFloor,
    BasicStone1,
    BasicWater1,
    BasicWater2,
}



pub fn spawn_resource_map() -> HashMap<ResourceId, &'static str> {
    [
        (ResourceId::StoneFloor, "images/StoneFloor.png"),
        (ResourceId::BasicStone1, "images/BasicStone1.png"),
        (ResourceId::BasicWater1, "images/BasicWater1.png"),

    ].iter().cloned().collect()
}