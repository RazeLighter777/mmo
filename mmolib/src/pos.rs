use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    component::{self, ComponentInterface},
    registry,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pos {
    pub x: u64,
    pub y: u64,
}
impl component::ComponentDataType for Pos {}
