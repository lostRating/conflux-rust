// Copyright 2020 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

#[cfg(test)]
impl KeyValueDbTraitOwnedRead for KvdbRocksdb {
    fn get_mut(&mut self, _key: &[u8]) -> Result<Option<Box<[u8]>>> {
        unreachable!()
    }
}

#[cfg(test)]
impl<'db> OpenSnapshotMptTrait<'db> for KvdbRocksdb {
    type SnapshotDbAsOwnedType = SnapshotMpt<
        KvdbSqliteSharded<SnapshotMptDbValue>,
        KvdbSqliteSharded<SnapshotMptDbValue>,
    >;
    type SnapshotDbBorrowMutType = SnapshotMpt<
        KvdbSqliteSharded<SnapshotMptDbValue>,
        KvdbSqliteSharded<SnapshotMptDbValue>,
    >;
    type SnapshotDbBorrowSharedType = SnapshotMpt<
        KvdbSqliteSharded<SnapshotMptDbValue>,
        KvdbSqliteSharded<SnapshotMptDbValue>,
    >;

    fn open_snapshot_mpt_owned(
        &'db mut self,
    ) -> Result<Self::SnapshotDbBorrowMutType> {
        unreachable!();
    }

    fn open_snapshot_mpt_as_owned(
        &'db self,
    ) -> Result<Self::SnapshotDbAsOwnedType> {
        unreachable!();
    }

    fn open_snapshot_mpt_shared(
        &'db self,
    ) -> Result<Self::SnapshotDbBorrowSharedType> {
        unreachable!();
    }
}

#[cfg(test)]
impl KeyValueDbTraitSingleWriter for KvdbRocksdb {
    fn delete(&mut self, _key: &[u8]) -> Result<Option<Option<Box<[u8]>>>> {
        unreachable!();
    }

    fn put(
        &mut self, key: &[u8], value: &[u8],
    ) -> Result<Option<Option<Box<[u8]>>>> {
        let mut transaction = self.kvdb.transaction();
        transaction.put(self.col, key, value);
        self.kvdb.write(transaction)?;
        Ok(None)
    }
}

#[cfg(test)]
impl SnapshotDbTrait for KvdbRocksdb {
    type SnapshotKvdbIterTraitTag = KvdbSqliteShardedIteratorTag;
    type SnapshotKvdbIterType =
        KvdbSqliteSharded<Box<[u8]>>;

    fn get_null_snapshot() -> Self {
        unreachable!();
    }

    fn open(
        snapshot_path: &Path, _readonly: bool,
        already_open_snapshots: &AlreadyOpenSnapshots<Self>,
        open_semaphore: &Arc<Semaphore>,
    ) -> Result<KvdbRocksdb>
    {
        Self::create(
            snapshot_path,
            already_open_snapshots,
        open_semaphore)
    }

    fn create(
        snapshot_path: &Path,
        _already_open_snapshots: &AlreadyOpenSnapshots<Self>,
        _open_snapshots_semaphore: &Arc<Semaphore>,
    ) -> Result<KvdbRocksdb>
    {
        Ok(KvdbRocksdb {
            kvdb: Arc::new(Database::open_default(
                snapshot_path.to_str().unwrap(),
            )?),
            col: 0,
        })
    }

    fn direct_merge(&mut self) -> Result<MerkleHash> {
        unreachable!();
    }

    fn copy_and_merge(
        &mut self, _old_snapshot_db: &KvdbRocksdb,
    ) -> Result<MerkleHash> {
        unreachable!();
    }

    fn start_transaction(&mut self) -> Result<()> {
        Ok(())
    }

    fn commit_transaction(&mut self) -> Result<()> {
        Ok(())
    }

    fn snapshot_kv_iterator(
        &self,
    ) -> Result<
        Wrap<
            KvdbSqliteSharded<Box<[u8]>>,
            dyn KeyValueDbIterableTrait<
                MptKeyValue,
                [u8],
                KvdbSqliteShardedIteratorTag,
            >,
        >,
    > {
        unreachable!();
    }
}

trait A : Sized {}

#[cfg(test)]
#[test]
pub fn test_db_size() {
    fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
        fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
            dir.try_fold(0, |acc, file| {
                let file = file?;
                let size = match file.metadata()? {
                    data if data.is_dir() => {
                        dir_size(fs::read_dir(file.path())?)?
                    }
                    data => data.len(),
                };
                Ok(acc + size)
            })
        }

        dir_size(fs::read_dir(path.into())?)
    }

    const PATH_STR: &'static str = "./tmp/";
    let sql_path = PATH_STR.to_string() + "sql/";
    let kv_keys_per_round = 1000000;
    let code_keys_per_found = 10000;
    let exp_params = vec![
        //("snapshot_db", "kv", 10, kv_keys_per_round),
        ("snapshot_db", "kv", 30, kv_keys_per_round),
        ("snapshot_db", "kv", 50, kv_keys_per_round),
        ("snapshot_db", "code", 5_000, code_keys_per_found),
        ("snapshot_db", "code", 10_000, code_keys_per_found),
        ("snapshot_db", "code", 15_000, code_keys_per_found),
        ("snapshot_db", "code", 20_000, code_keys_per_found),
    ];
    const ROUNDS: u32 = 10;

    for exp_params in &exp_params {
        let (db_type, exp_type, data_len, kvs_per_round) = exp_params;
        println!(
            "start experiment {} data len {} kvs_per_round {}",
            exp_type, data_len, kvs_per_round
        );
        let already_open_snapshots: AlreadyOpenSnapshots<SnapshotDbSqlite> =
            Default::default();
        let open_snapshot_semaphore: Arc<Semaphore> =
            Arc::new(Semaphore::new(1 as usize));
            let db = SnapshotDbSqlite::create(
                Path::new(&sql_path),
                &already_open_snapshots,
                &open_snapshot_semaphore,
            )
            .unwrap();
        drop(db);
        println!("init {}", dir_size(&PATH_STR).unwrap());
        for _i in 0..ROUNDS {
            let mut db = SnapshotDbSqlite::create(
                Path::new(&sql_path),
                &already_open_snapshots,
                &open_snapshot_semaphore,
            )
            .unwrap();
            db.start_transaction().unwrap();
            for _j in 0..*kvs_per_round {
                let address = Address::random();
                let key_suffix = H256::random();
                let key;
                let code_value;
                let value_value;
                let value_ref;
                if *exp_type == "code" {
                    key = StorageKey::new_code_key(
                        &address,
                        /* code_hash = */ &key_suffix,
                    )
                    .to_key_bytes();
                    let code = random_string(*data_len).as_bytes().to_vec();
                    let code_info = CodeInfo {
                        code: Arc::new(code),
                        owner: address,
                    };
                    code_value = ::rlp::encode(&code_info);
                    value_ref = code_value.as_ref();
                } else {
                    key = StorageKey::new_storage_key(
                        &address,
                        key_suffix.as_ref(),
                    )
                    .to_key_bytes();
                    value_value = random_string(*data_len);
                    value_ref = value_value.as_bytes();
                };

                db.put(&key, &value_ref).expect("insert kv");
            }
            db.commit_transaction().unwrap();
            drop(db);
            println!("round {}: {}", _i, dir_size(&PATH_STR).unwrap());
        }
        fs::remove_dir_all(PATH_STR).expect("remove dir");
    }
}

#[cfg(test)]
pub fn open_snapshot_db_for_testing(
    snapshot_path: &Path, readonly: bool,
) -> Result<SnapshotDbSqlite> {
    SnapshotDbSqlite::open(
        snapshot_path,
        readonly,
        &Default::default(),
        &Arc::new(Semaphore::new(DEFAULT_MAX_OPEN_SNAPSHOTS as usize)),
    )
}

pub trait MptValueKind: Debug {
    fn value_eq(&self, maybe_value: Option<&[u8]>) -> bool;
}

impl MptValueKind for () {
    fn value_eq(&self, maybe_value: Option<&[u8]>) -> bool {
        maybe_value.is_none()
    }
}

impl MptValueKind for Box<[u8]> {
    fn value_eq(&self, maybe_value: Option<&[u8]>) -> bool {
        maybe_value.map_or(false, |v| v.eq(&**self))
    }
}

pub fn check_key_value_load<Value: MptValueKind>(
    snapshot_db: &SnapshotDbSqlite,
    mut kv_iter: impl FallibleIterator<Item = (Vec<u8>, Value), Error = Error>,
    check_value: bool,
) -> Result<u64>
{
    let mut checker_count = 0;
    let mut mpt = snapshot_db.open_snapshot_mpt_shared()?;

    let mut cursor = MptCursor::<
        &mut dyn SnapshotMptTraitRead,
        BasicPathNode<&mut dyn SnapshotMptTraitRead>,
    >::new(&mut mpt);
    cursor.load_root()?;
    while let Some((access_key, expected_value)) = kv_iter.next()? {
        let terminal =
            cursor.open_path_for_key::<access_mode::Read>(&access_key)?;
        if check_value {
            let mpt_value = match terminal {
                CursorOpenPathTerminal::Arrived => {
                    cursor.current_node_mut().value_as_slice().into_option()
                }
                CursorOpenPathTerminal::ChildNotFound { .. } => None,
                CursorOpenPathTerminal::PathDiverted(_) => None,
            };
            if !expected_value.value_eq(mpt_value) {
                error!(
                    "mpt value doesn't match snapshot kv. Expected {:?}, got {:?}",
                    expected_value, mpt_value,
                );
            }
        }
        checker_count += 1;
    }
    cursor.finish()?;

    Ok(checker_count)
}

use crate::{
    impls::{
        errors::*,
        merkle_patricia_trie::{
            mpt_cursor::{BasicPathNode, CursorOpenPathTerminal, MptCursor},
            TrieNodeTrait,
        },
        storage_db::{
            kvdb_sqlite_sharded::KvdbSqliteSharded,
            snapshot_db_sqlite::SnapshotDbSqlite,
        }
    },
    storage_db::{snapshot_db::OpenSnapshotMptTrait, SnapshotMptTraitRead},
    utils::access_mode,
};
use fallible_iterator::FallibleIterator;
use std::fmt::Debug;

#[cfg(test)]
use crate::impls::{
    defaults::DEFAULT_MAX_OPEN_SNAPSHOTS,
    merkle_patricia_trie::MptKeyValue,
    storage_db::{
        kvdb_rocksdb::KvdbRocksdb,
        kvdb_sqlite_sharded::KvdbSqliteShardedIteratorTag,
        snapshot_db_manager_sqlite::AlreadyOpenSnapshots,
        snapshot_db_sqlite::SnapshotDbTrait,
        snapshot_mpt::SnapshotMpt,
    },
};
#[cfg(test)]
use crate::{
    storage_db::{
        KeyValueDbIterableTrait,
        KeyValueDbTraitOwnedRead,
        KeyValueDbTraitSingleWriter,
        KeyValueDbTypes,
        snapshot_mpt::SnapshotMptDbValue,
    },
    utils::wrap::Wrap,
};
#[cfg(test)]
use cfx_types::{Address, H256};
#[cfg(test)]
use cfxstore::random_string;
#[cfg(test)]
use kvdb_rocksdb::Database;
#[cfg(test)]
use primitives::{CodeInfo, MerkleHash, StorageKey};
#[cfg(test)]
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};
#[cfg(test)]
use tokio::sync::Semaphore;
