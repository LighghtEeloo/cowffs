use crate::FileSys;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Index, IndexMut},
    sync::{Arc, RwLock},
};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockId(pub(crate) usize);

impl Index<BlockId> for FileSys {
    type Output = Arc<RwLock<Block>>;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

impl IndexMut<BlockId> for FileSys {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Block {
    INode(INode),
    DirEntries(Vec<Option<DirEntry>>),
    Data(Data),
}

impl Block {
    pub fn inode(self) -> INode {
        match self {
            Block::INode(inode) => inode,
            _ => panic!("Not an INode"),
        }
    }
    pub fn dir_entries(self) -> Vec<Option<DirEntry>> {
        match self {
            Block::DirEntries(dir_entries) => dir_entries,
            _ => panic!("Not a DirEntries"),
        }
    }
    pub fn data(self) -> Data {
        match self {
            Block::Data(data) => data,
            _ => panic!("Not a Data"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum BlockType {
    File,
    Dir,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DirEntry {
    pub name: String,
    // pub owner: String,
    // pub read: bool,
    // pub write: bool,
    // pub execute: bool,
    pub btype: BlockType,
    pub inode: BlockId,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct INode {
    pub ref_cnt: usize,
    pub children: Vec<BlockId>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub data: Vec<u8>,
}
