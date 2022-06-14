use serde::{Deserialize, Serialize};

use crate::effect;
use crate::{raws::Raw, resource};
pub type BlockTypeId = u64;
#[derive(Deserialize, Clone, Debug)]
#[repr(u8)]
pub enum BlockLayer {
    Ground = 0,
    Solid = 1,
    Water = 2,
    Pit = 3,
    Effect(effect::Effect) = 5,
}

#[derive(Deserialize, Debug)]
pub struct BlockType {
    canonical_name: String,
    descriptive_name: String,
    layer: BlockLayer,
    resource: resource::ResourceId,
}

impl BlockType {
    pub fn new(raw: &Raw) -> Result<BlockType, serde_json::Error> {
        let res: BlockType = serde_json::from_value(raw.dat().clone())?;
        Ok(res)
    }
    pub fn get_canonical_name(&self) -> &str {
        &self.canonical_name
    }
    pub fn get_descriptive_name(&self) -> &str {
        &self.descriptive_name
    }
    pub fn get_layer(&self) -> BlockLayer {
        self.layer.clone()
    }
}
