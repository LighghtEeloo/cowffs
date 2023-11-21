#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use state_machines_macros::{case_on_init, case_on_next, state_machine};
use vstd::{map::*, pervasive::*, seq::*, set::*, *};

verus! {

#[verifier(external_body)] /* vattr */
pub struct Path(pub String);

pub struct MetaData {
    pub idx: Seq<usize>,
    pub typ: MetaType,
}

pub enum MetaType {
    File,
    Dir,
    Link,
}

pub enum Block {
    DirEntries(Seq<Path>),
    Data(DataBlock),
}

#[verifier(external_body)] /* vattr */
pub struct DataBlock { }

#[verifier(external_body)] /* vattr */
#[verifier::spec]
pub fn datablock() -> DataBlock { unimplemented!() }

state_machine! {
    FileSysSpec {
        fields {
            pub metadata: Map<Path, Option<MetaData>>,
            pub blocks: Seq<DataBlock>,
            pub block_cnt: nat,
        }

        init! {
            empty() {
                init metadata = Map::total(|p| None);
                init blocks = Seq::new(0, |i| datablock());
                init block_cnt = 0;
            }
        }

        transition! {
            create_file_op(path: Path) {
                update metadata = pre.metadata.insert(path, Some(MetaData {
                    idx: Seq::new(0, |i| 0), typ: MetaType::File,
                }));
            }
        }

        transition! {
            read_file_op(path: Path, idx: Seq<usize>) {
                require(pre.metadata.contains_pair(path, Some(MetaData {
                    idx, typ: MetaType::File,
                })));
            }
        }

        transition! {
            noop() {
            }
        }
    }
}

state_machine! {
    ShardedKVProtocol {
        fields {
            pub thread_cnt: int,
            pub maps: Seq<Map<Path, Option<MetaData>>>,
        }

        init! {
            initialize(thread_cnt: int) {
                require(0 < thread_cnt);
                init thread_cnt = thread_cnt;
                init maps = Seq::new(thread_cnt as nat, |i| {
                    if i == 0 {
                        Map::total(|k| None)
                    } else {
                        Map::empty()
                    }
                });
            }
        }

        pub open spec fn valid_host(&self, i: int) -> bool {
            0 <= i < self.thread_cnt
        }

        transition! {
            insert(idx: int, key: Path, value: Option<MetaData>) {
                require(pre.valid_host(idx));
                require(pre.maps.index(idx).dom().contains(key));
                update maps[idx][key] = value;
            }
        }

        transition! {
            query(idx: int, key: Path, value: Option<MetaData>) {
                require(pre.valid_host(idx));
                require(pre.maps.index(idx).contains_pair(key, value));
            }
        }

        transition! {
            transfer(send_idx: int, recv_idx: int, key: Path, value: Option<MetaData>) {
                require(pre.valid_host(send_idx));
                require(pre.valid_host(recv_idx));
                require(pre.maps.index(send_idx).contains_pair(key, value));
                require(send_idx != recv_idx);
                update maps[send_idx] = pre.maps.index(send_idx).remove(key);
                update maps[recv_idx][key] = value;
            }
        }

        pub open spec fn host_has_key(&self, hostidx: int, key: Path) -> bool {
            self.valid_host(hostidx)
            && self.maps.index(hostidx).dom().contains(key)
        }

        pub open spec fn key_holder(&self, key: Path) -> int {
            choose|idx| self.host_has_key(idx, key)
        }

        pub open spec fn abstraction_one_key(&self, key: Path) -> Option<MetaData> {
            if exists |idx| self.host_has_key(idx, key) {
                self.maps.index(self.key_holder(key)).index(key)
            } else {
                None
            }
        }

        pub open spec fn interp_map(&self) -> Map<Path, Option<MetaData>> {
            Map::total(|key| self.abstraction_one_key(key))
        }

        #[invariant]
        pub open fn num_hosts(&self) -> bool {
            self.maps.len() == self.thread_cnt
        }

        #[invariant]
        pub open fn inv_no_dupes(&self) -> bool {
            forall |i: int, j: int, key: Path|
                self.host_has_key(i, key) && self.host_has_key(j, key) ==> i == j
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self, thread_cnt: int) {
        }
       
        #[inductive(insert)]
        fn insert_inductive(pre: Self, post: Self, idx: int, key: Path, value: Option<MetaData>) {
            //assert(forall(|k: Path| pre.host_has_key(idx, k) ==> post.host_has_key(idx, k)));
            //assert(forall(|k: Path| post.host_has_key(idx, k) ==> pre.host_has_key(idx, k)));
            //assert(forall(|k: Path| pre.host_has_key(idx, k) == post.host_has_key(idx, k)));
            assert(forall |i: int, k: Path| pre.host_has_key(i, k) == post.host_has_key(i, k));
        }
       
        #[inductive(query)]
        fn query_inductive(pre: Self, post: Self, idx: int, key: Path, value: Option<MetaData>) { }
       
        #[inductive(transfer)]
        fn transfer_inductive(pre: Self, post: Self, send_idx: int, recv_idx: int, key: Path, value: Option<MetaData>) {
            assert(forall |i: int, k: Path| !equal(k, key) ==> pre.host_has_key(i, k) == post.host_has_key(i, k));
            assert(forall |i: int| i != send_idx && i != recv_idx ==> pre.host_has_key(i, key) == post.host_has_key(i, key));

            assert(equal(post.maps.index(send_idx),
                pre.maps.index(send_idx).remove(key)));

            assert(!post.host_has_key(send_idx, key));
            assert(pre.host_has_key(send_idx, key));

            /*assert_forall_by(|i: int, j: int, k: Path| {
                requires(post.host_has_key(i, k) && post.host_has_key(j, k));
                ensures(i == j);
                if equal(k, key) {
                    assert(i != send_idx);
                    assert(j != send_idx);
                    if i != recv_idx {
                        assert(pre.host_has_key(i, key));
                    }
                    if i != recv_idx && j != recv_idx {
                        assert(pre.host_has_key(i, key));
                        assert(pre.host_has_key(j, key));
                        assert(pre.inv_no_dupes());
                        assert(i == j);
                    }
                    assert(i == j);
                } else {
                    assert(i == j);
                }
            });*/
        }
    }
}

fn main() { }

}
