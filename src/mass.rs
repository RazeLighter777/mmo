use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{component::{self, ComponentInterface}, context, registry};

#[derive(Serialize, Deserialize, Debug)]
pub struct Mass {
    pub m: u64,
}
impl component::ComponentDataType for Mass {
    fn post_deserialization(&mut self, context : Arc<context::Context>) -> Vec<Box<dyn ComponentInterface>> {
        Vec::new()
    }
}
