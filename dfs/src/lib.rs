#![allow(unused)]

use dashmap::DashMap;
use refffs::{Data, DirEntries};
use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{Arc, RwLock, mpsc},
    thread,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockPtr(usize);

#[derive(Clone, Debug)]
pub enum Block {
    Data(Data),
    Dir(DirEntries<BlockPtr>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WorkerId(usize);

pub struct Worker {
    shard: HashMap<BlockPtr, Block>,
}

pub struct FileSystem {
    shards: Vec<HashSet<BlockPtr>>,
    // workers: Vec<Worker>,
}

impl FileSystem {
    pub fn new(worker_num: usize) -> io::Result<Self> {
        let t = thread::Builder::new().name("worker".to_string()).spawn(|| {
            let worker = Worker {
                shard: Default::default(),
            };
        })?;

        Ok(Self {
            shards: Default::default(),
        })
    }
}
