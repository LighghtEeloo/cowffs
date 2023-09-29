pub mod view;
pub mod block;
pub mod stepper;

use block::{Block, BlockId, BlockType, DirEntry, INode};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use stepper::Stepper;
use view::FPath;

pub struct FileSys {
    pub instance: PathBuf,
    pub blocks: Vec<Arc<RwLock<Block>>>,
}

impl FileSys {
    pub fn fs_new_blocks() -> Vec<Block> {
        vec![Block::INode(INode {
            ref_cnt: 1,
            children: vec![],
        })]
    }
    pub fn fs_read_or_create(instance: PathBuf) -> anyhow::Result<FileSys> {
        let blocks = match std::fs::read_to_string(&instance) {
            Ok(s) => serde_json::from_str(&s)?,
            Err(_) => FileSys::fs_new_blocks(),
        };
        let blocks = blocks
            .into_iter()
            .map(|block| Arc::new(RwLock::new(block)))
            .collect::<Vec<_>>();
        Ok(FileSys { instance, blocks })
    }
    pub fn fs_write(&self) -> anyhow::Result<()> {
        let blocks = self
            .blocks
            .iter()
            .map(|block| block.read().unwrap().clone())
            .collect::<Vec<_>>();
        let s = serde_json::to_string(&blocks)?;
        print!(
            "Instance located at `{}` will be rewritten. Proceed? [../^C]",
            self.instance.display()
        );
        std::io::stdin().read_line(&mut String::new())?;
        std::fs::write(&self.instance, s)?;
        Ok(())
    }
    pub fn root() -> BlockId {
        BlockId(0)
    }
    pub fn create(&mut self, path: String) -> anyhow::Result<()> {
        let mut stepper = Stepper::new(self, FPath::new(path.as_str())?)?;
        Ok(())
    }
}
