use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    component::{self, ComponentInterface},
    registry,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}
impl component::ComponentDataType for Pos {}
