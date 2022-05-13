use serde::{Serialize, Deserialize};

use crate::component;


#[derive(Serialize, Deserialize, Debug)]
pub struct Mass {
    pub m : u64
}
impl component::ComponentDataType for Mass {

}