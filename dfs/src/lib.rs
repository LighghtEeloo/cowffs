#![allow(unused)]

use dashmap::DashMap;
use refffs::{Data, DirEntries};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
    thread,
};
pub struct BlockPtr(usize);

pub enum Block {
    Data(Data),
    Dir(DirEntries<BlockPtr>),
}

pub struct WorkerId(usize);

pub struct Worker {
    shard: HashMap<BlockPtr, Block>,
}

pub struct FileSystem {
    // workers: Vec<Worker>,
}

impl FileSystem {
    pub fn new(worker_num: usize) -> io::Result<Self> {
        let t = thread::Builder::new().name("worker".to_string()).spawn(|| {
            let shard = HashMap::new();
            let worker = Worker { shard };
        })?;

        Ok(Self {})
    }
}
