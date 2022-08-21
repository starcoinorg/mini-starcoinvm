use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use std::cell::Cell;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

static mut STEPS: Cell<u32> = Cell::new(0u32);

pub fn skip_step() {
    unsafe {
        get_and_inc();
    }
}

pub fn init_file_path(block_hash: HashValue) -> std::io::Result<()> {
    fs::create_dir_all(block_hash.to_hex())
}

pub fn serialize_to_file<T, K>(block_hash: HashValue, key: &K, obj: &T) -> std::io::Result<()>
where
    T: ?Sized + Serialize + Debug,
    K: ?Sized + Serialize + Debug,
{
    let mut key_bytes = bcs_ext::to_bytes(key).unwrap();
    unsafe {
        append_nonce(&mut key_bytes);
    }
    let file_name = format!(
        "./{}/{}",
        block_hash.to_hex(),
        HashValue::sha3_256_of(key_bytes.as_slice()).to_hex()
    );
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
    block_hash: HashValue,
    key: &K,
) -> anyhow::Result<HashValue>
where
    K: ?Sized + Serialize + Debug,
{
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).unwrap().as_slice()))
}

pub fn deserialize_from_file_for_block<K>(block_hash: HashValue, key: &K) -> anyhow::Result<Block>
where
    K: ?Sized + Serialize + Debug,
{
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).unwrap().as_slice()))
}

pub fn deserialize_from_file_for_vev_u8<K>(
    block_hash: HashValue,
    key: &K,
) -> anyhow::Result<Option<Vec<u8>>>
where
    K: ?Sized + Serialize + Debug,
{
    let serialized_op = read_from_file(block_hash, key);
    match serialized_op {
        None => Ok(None),
        Some(serialized) => match bcs_ext::from_bytes(&(serialized.as_slice())) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        },
    }
}

fn read_from_file<'a, K>(block_hash: HashValue, key: &K) -> Option<Vec<u8>>
where
    K: ?Sized + Serialize + Debug,
{
    let mut key_bytes = bcs_ext::to_bytes(key).unwrap();
    unsafe {
        append_nonce(&mut key_bytes);
    }
    let file_name = format!(
        "./{}/{}",
        block_hash.to_hex(),
        HashValue::sha3_256_of(key_bytes.as_slice()).to_hex()
    );
    let file_op = File::options().read(true).open(file_name);

    match file_op {
        Ok(file) => {
            let mut read = BufReader::new(file);
            let mut serialized = vec![];
            read.read_to_end(&mut serialized).unwrap();
            Some(serialized)
        }
        Err(_) => None,
    }
}

unsafe fn get_and_inc() -> u32 {
    let step = STEPS.get();
    STEPS.set(step + 1);
    step
}

unsafe fn append_nonce(u: &mut Vec<u8>) {
    u.append(&mut get_and_inc().to_be_bytes().to_vec());
}
