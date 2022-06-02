use crate::{raws::RawTree, registry};

pub struct Context<'a> {
    pub raws: &'a RawTree,
    pub registry: &'a registry::Registry,
}
