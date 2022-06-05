use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    component::{self, ComponentInterface},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Mass {
    pub m: u64,
}
impl component::ComponentDataType for Mass {}
