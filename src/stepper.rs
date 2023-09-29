use crate::{
    block::{Block, BlockId},
    view::{self, FPath},
    FileSys,
};
use std::sync::{Arc, RwLock};

pub struct Stepper<'fp> {
    pub lock: Arc<RwLock<Block>>,
    pub current: BlockId,
    pub fp_iter: view::IntoIter<'fp>,
}

impl<'fp> Stepper<'fp> {
    pub fn new(fs: &mut FileSys, fpath: FPath<'fp>) -> anyhow::Result<Self> {
        let current = FileSys::root();
        Ok(Stepper {
            lock: fs[current].clone(),
            current,
            fp_iter: fpath.into_iter(),
        })
    }
}
