use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub struct FileHelper {
    base_path: String,
}

impl FileHelper {
    pub fn new(base_path: String) -> FileHelper {
        FileHelper { base_path }
    }

    pub fn init_file_path(&self, block_hash: HashValue) -> std::io::Result<()> {
        fs::create_dir_all(self.to_path(vec![block_hash.to_hex()]))
    }

    pub fn serialize_to_file<T, K>(
        &self,
        block_hash: HashValue,
        key: &K,
        obj: &T,
    ) -> std::io::Result<()>
    where
        T: ?Sized + Serialize + Debug,
        K: ?Sized + Serialize + Debug,
    {
        let key_bytes = bcs_ext::to_bytes(key).unwrap();
        let file_name = self.to_path(vec![
            block_hash.to_hex(),
            HashValue::sha3_256_of(key_bytes.as_slice()).to_hex(),
        ]);
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_name)
            .unwrap();
        let mut writer = BufWriter::new(file);

        let serialized = bcs_ext::to_bytes(obj).unwrap();
        writer.write_all(&serialized)
    }

    pub fn deserialize_from_file_for_block_state_root<K>(
        &self,
        block_hash: HashValue,
        key: &K,
    ) -> anyhow::Result<HashValue>
    where
        K: ?Sized + Serialize + Debug,
    {
        bcs_ext::from_bytes(&(self.read_from_file(block_hash, key).unwrap().as_slice()))
    }

    pub fn deserialize_from_file_for_block<K>(
        &self,
        block_hash: HashValue,
        key: &K,
    ) -> anyhow::Result<Block>
    where
        K: ?Sized + Serialize + Debug,
    {
        bcs_ext::from_bytes(&(self.read_from_file(block_hash, key).unwrap().as_slice()))
    }

    pub fn deserialize_from_file_for_vev_u8<K>(
        &self,
        block_hash: HashValue,
        key: &K,
    ) -> anyhow::Result<Option<Vec<u8>>>
    where
        K: ?Sized + Serialize + Debug,
    {
        let serialized_op = self.read_from_file(block_hash, key);
        match serialized_op {
            None => Ok(None),
            Some(serialized) => match bcs_ext::from_bytes(&(serialized.as_slice())) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e),
            },
        }
    }

    fn read_from_file<'a, K>(&self, block_hash: HashValue, key: &K) -> Option<Vec<u8>>
    where
        K: ?Sized + Serialize + Debug,
    {
        let key_bytes = bcs_ext::to_bytes(key).unwrap();
        let file_path = self.to_path(vec![
            block_hash.to_hex(),
            HashValue::sha3_256_of(key_bytes.as_slice()).to_hex(),
        ]);

        match File::options().read(true).open(file_path) {
            Ok(file) => {
                let mut read = BufReader::new(file);
                let mut serialized = vec![];
                read.read_to_end(&mut serialized).unwrap();
                Some(serialized)
            }
            Err(_) => None,
        }
    }

    fn to_path(&self, p_v: Vec<String>) -> PathBuf {
        let mut path = PathBuf::new();
        path = path.join(self.base_path.clone());
        for p in p_v {
            path = path.join(p);
        }
        path
    }
}
