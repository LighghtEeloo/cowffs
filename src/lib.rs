use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct FileSys {
    pub instance: PathBuf,
    pub blocks: Vec<Block>,
}

pub struct FPath<'a>(pub Vec<&'a str>);
impl<'a> FPath<'a> {
    pub fn new(s: &'a str) -> anyhow::Result<Self> {
        if s.starts_with('/') {
            let x = s[1..]
                .split('/')
                .map(|s| {
                    if s.is_empty() {
                        Err(anyhow::anyhow!("Path must not contain empty components"))
                    } else {
                        Ok(s)
                    }
                })
                .collect::<Result<_, _>>()?;
            Ok(FPath(x))
        } else {
            Err(anyhow::anyhow!("Path must start with `/`"))
        }
    }
}
impl<'a> IntoIterator for FPath<'a> {
    type Item = &'a str;

    type IntoIter = std::vec::IntoIter<&'a str>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
// pub struct Iter<'a>(std::slice::Iter<'a, &'a str>);
// impl<'a> FPath<'a> {
//     pub fn iter<'b: 'a>(&'b self) -> Iter<'a> {
//         Iter(self.0.iter())
//     }
// }

// impl<'a> Iterator for Iter<'a> {
//     type Item = &'a str;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.next().map(|p| *p)
//     }
// }

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct BlockId(usize);

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub ref_cnt: usize,
    pub inner: BlockInner,
}

#[derive(Serialize, Deserialize)]
pub enum BlockInner {
    DirEntries(Vec<DirEntry>),
    INode(INode),
    Data(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub block: usize,
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
    pub fn fs_new_blocks() -> Vec<Block> {
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
    pub fn fs_get_block(&self, id: BlockId) -> anyhow::Result<&Block> {
        self.blocks.get(id.0).ok_or(anyhow::anyhow!("Block not found"))
    }
    pub fn fs_get_block_mut(&mut self, id: BlockId) -> anyhow::Result<&mut Block> {
        self.blocks.get_mut(id.0).ok_or(anyhow::anyhow!("Block not found"))
    }
    pub fn create(&mut self, path: String) -> anyhow::Result<()> {
        for fp in FPath::new(path.as_str())? {}
        Ok(())
    }
}
