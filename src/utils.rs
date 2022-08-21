use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use std::cell::Cell;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

static mut NONCE: Cell<u8> = Cell::new(0u8);

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
        key_bytes.push(NONCE.get());
        NONCE.set(NONCE.get() + 1);
        println!("nonce: {:?}", NONCE.get());
    }
    let file_name = format!(
        "./{}/{}",
        block_hash.to_hex(),
        HashValue::sha3_256_of(key_bytes.as_slice()).to_hex()
    );
    println!(
        "write file_name: {:?}, key: {:?}, obj: {:?}",
        file_name, key, obj
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
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).as_slice()))
}

pub fn deserialize_from_file_for_block<K>(block_hash: HashValue, key: &K) -> anyhow::Result<Block>
where
    K: ?Sized + Serialize + Debug,
{
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).as_slice()))
}

pub fn deserialize_from_file_for_state_node<K>(
    block_hash: HashValue,
    key: &K,
) -> anyhow::Result<Vec<u8>>
where
    K: ?Sized + Serialize + Debug,
{
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).as_slice()))
}

pub fn deserialize_from_file_for_access_path<K>(
    block_hash: HashValue,
    key: &K,
) -> anyhow::Result<Vec<u8>>
where
    K: ?Sized + Serialize + Debug,
{
    bcs_ext::from_bytes(&(read_from_file(block_hash, key).as_slice()))
}

fn read_from_file<'a, K>(block_hash: HashValue, key: &K) -> Vec<u8>
where
    K: ?Sized + Serialize + Debug,
{
    let mut key_bytes = bcs_ext::to_bytes(key).unwrap();
    unsafe {
        key_bytes.push(NONCE.get());
        NONCE.set(NONCE.get() + 1);
        println!("nonce: {:?}", NONCE.get());
    }
    let file_name = format!(
        "./{}/{}",
        block_hash.to_hex(),
        HashValue::sha3_256_of(key_bytes.as_slice()).to_hex()
    );
    println!("read file_name: {:?}, key: {:?}", file_name, key);
    let file = File::options().read(true).open(file_name).unwrap();

    let mut read = BufReader::new(file);
    let mut serialized = vec![];
    read.read_to_end(&mut serialized).unwrap();
    serialized
}
