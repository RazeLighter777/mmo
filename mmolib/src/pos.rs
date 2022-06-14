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
}
impl component::ComponentDataType for Pos {}
