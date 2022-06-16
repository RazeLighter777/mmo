use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::component::{self, ComponentInterface};

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub username: String,
}
impl component::ComponentDataType for Player {}
