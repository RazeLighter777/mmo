use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    chunk,
    component::{self, ComponentInterface},
    registry,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pos {
    pub pos: chunk::Position,
    pub load_with_chunk: bool,
}
impl component::ComponentDataType for Pos {}
