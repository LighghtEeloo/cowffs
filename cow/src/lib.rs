#![allow(unused)]

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
    fn fs_new_blocks() -> Vec<Block> {
        vec![Block::INode(INode {
            ref_cnt: 1,
            children: vec![],
        })]
    }
    fn fs_disk_init_with_blocks(instance: PathBuf, blocks: Vec<Block>) -> anyhow::Result<FileSys> {
        let blocks = blocks
            .into_iter()
            .map(|block| Arc::new(RwLock::new(block)))
            .collect::<Vec<_>>();
        Ok(FileSys { instance, blocks })
    }
    pub fn fs_disk_init(instance: PathBuf) -> anyhow::Result<FileSys> {
        let blocks = FileSys::fs_new_blocks();
        Self::fs_disk_init_with_blocks(instance, blocks)
    }
    pub fn fs_disk_load(instance: PathBuf) -> anyhow::Result<FileSys> {
        let blocks: Vec<_> = serde_json::from_str(&std::fs::read_to_string(&instance)?)?;
        Self::fs_disk_init_with_blocks(instance, blocks)
    }
    pub fn fs_disk_dump(&self) -> anyhow::Result<()> {
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
