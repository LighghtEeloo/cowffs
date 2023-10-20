#![allow(unused)]

use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Data(Vec<u8>);

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = String::new();
        for byte in &self.0 {
            data += &format!("{:02x}", byte);
        }
        write!(f, "{}", data)
    }
}

impl Data {
    pub fn new(raw: impl AsRef<[u8]>) -> Self {
        Self(raw.as_ref().to_vec())
    }
}

pub struct Disk(Vec<Data>);
impl std::ops::Index<usize> for Disk {
    type Output = Data;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct Machine {
    /// written_to_cache
    on: bool,
    /// written_to_disk
    sync: bool,
}

impl Default for Machine {
    fn default() -> Self {
        Self { on: false, sync: false }
    }
}

impl Machine {}

// pub struct AsyncDisk {}

// pub struct VirtualAsyncDisk {}

// pub struct TxnDisk {}

pub trait Transaction: Sized {
    /// Transaction
    type Txn;
    /// Index
    type Idx;

    fn init(mach: Machine, disks: Vec<Disk>) -> Self;
    fn disks(self) -> Vec<Disk>;
    fn begin_tx(&mut self);
    fn write_tx(&mut self, txn: Self::Txn);
    fn commit_tx(&mut self);
    fn read(&self, idx: Self::Idx) -> &Data;
    fn crash(self) -> Self {
        Self::init(Machine::default(), self.disks())
    }
}

pub mod multi_txn_disk {
    use super::*;

    /// Transaction
    #[derive(Clone)]
    pub struct Txn {
        pub device: usize,
        pub offset: usize,
        pub data: Data,
    }

    pub struct Idx {
        pub device: usize,
        pub offset: usize,
    }
}

/// Multiple Transactional Disk
pub struct MultiTxnDisk {
    mach: Machine,
    caches: Vec<Disk>,
    disks: Vec<Disk>,
    txn: Option<Vec<multi_txn_disk::Txn>>,
}

impl Transaction for MultiTxnDisk {
    type Txn = multi_txn_disk::Txn;
    type Idx = multi_txn_disk::Idx;

    fn init(mach: Machine, disks: Vec<Disk>) -> Self {
        Self {
            mach,
            caches: Vec::new(),
            disks,
            txn: None,
        }
    }
    fn disks(self) -> Vec<Disk> {
        self.disks
    }
    fn begin_tx(&mut self) {
        assert!(self.txn.is_none());
        self.txn = Some(Vec::new());
    }
    fn write_tx(&mut self, txn: Self::Txn) {
        assert!(self.txn.is_some());
        self.txn.as_mut().expect("should have began transaction").push(txn);
    }
    fn commit_tx(&mut self) {
        let iter: Vec<_> = self.txn.iter().cloned().flatten().collect();
        self.writev(iter.into_iter());
        self.txn = None;
    }
    fn read(&self, idx: Self::Idx) -> &Data {
        &self.caches[idx.device][idx.offset]
    }
}

impl MultiTxnDisk {
    pub fn writev(&mut self, iov: impl Iterator<Item = multi_txn_disk::Txn>) {
        // let on = ;
    }
}

// pub struct Stat {}

// pub struct InodeSpec {}

// pub struct RangeVirtualTxnDisk {}

// pub struct SyncDisk {}

// pub struct BitmapSpec {}

// pub struct InodePackSpec {}

// pub struct Allocator64 {}

// pub struct Allocator32 {}
