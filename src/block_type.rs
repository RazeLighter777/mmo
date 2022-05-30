use serde::{Serialize, Deserialize};

use crate::{resource, raws::Raw};

pub type BlockTypeId = u16;
#[derive(Deserialize,Clone)]
pub enum BlockLayer {
    Ground = 0,
    Solid = 1,
    Water = 2,
    Pit = 3,
}

#[derive(Deserialize)]
pub struct BlockType {
    canonical_name : String,
    descriptive_name : String,
    raw_path : String,
    layer : BlockLayer,
}


impl BlockType {
    pub fn new(raw : &Raw) -> Result<BlockType, serde_json::Error> {
        let res : BlockType = serde_json::from_value(raw.dat().clone())?;
        Ok(res)
    }
    pub fn get_canonical_name(&self) -> &str {
        &self.canonical_name
    }
    pub fn get_descriptive_name(&self) -> &str {
        &self.descriptive_name
    }
    pub fn get_raw_path(&self) -> &str {
        &self.raw_path
    }
    pub fn get_layer(&self) -> BlockLayer {
        self.layer.clone()
    }
}