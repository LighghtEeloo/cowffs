use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct FileSys {
    pub instance: PathBuf,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub ref_cnt: usize,
    pub inner: BlockInner,
}

#[derive(Serialize, Deserialize)]
pub enum BlockInner {
    INode(INode),
    Data(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub struct INode {
    pub owner: String,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub children: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub data: Vec<u8>,
}

impl FileSys {
    pub fn new_blocks() -> Vec<Block> {
        vec![Block {
            ref_cnt: 1,
            inner: BlockInner::INode(INode {
                owner: "root".to_string(),
                read: true,
                write: true,
                execute: false,
                children: vec![],
            }),
        }]
    }
    pub fn read_or_create(instance: PathBuf) -> anyhow::Result<FileSys> {
        let blocks = match std::fs::read_to_string(&instance) {
            Ok(s) => serde_json::from_str(&s)?,
            Err(_) => FileSys::new_blocks(),
        };
        Ok(FileSys { instance, blocks })
    }
    pub fn write(&self) -> anyhow::Result<()> {
        let s = serde_json::to_string(&self.blocks)?;
        print!("Instance located at `{}` will be rewritten. Proceed? [../^C]", self.instance.display());
        std::io::stdin().read_line(&mut String::new())?;
        std::fs::write(&self.instance, s)?;
        Ok(())
    }
    // pub fn create(&mut self, path: PathBuf) {}
}
