pub mod view;
pub mod block;

use block::{Block, BlockId, BlockType, DirEntry, INode};
use std::path::PathBuf;
use view::FPath;

pub struct FileSys {
    pub instance: PathBuf,
    pub blocks: Vec<Block>,
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
        Ok(FileSys { instance, blocks })
    }
    pub fn fs_write(&self) -> anyhow::Result<()> {
        let s = serde_json::to_string(&self.blocks)?;
        print!(
            "Instance located at `{}` will be rewritten. Proceed? [../^C]",
            self.instance.display()
        );
        std::io::stdin().read_line(&mut String::new())?;
        std::fs::write(&self.instance, s)?;
        Ok(())
    }
    pub fn create(&mut self, path: String) -> anyhow::Result<()> {
        for name in FPath::new(path.as_str())? {}
        Ok(())
    }
}
