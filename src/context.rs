use crate::raws::RawTree;

pub struct Context<'a> {
    pub raws: &'a RawTree,
}
