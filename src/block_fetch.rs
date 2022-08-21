use crate::{types, utils};
use bcs_ext;
use starcoin_crypto::HashValue;
use starcoin_rpc_client::StateRootOption::BlockHash;
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_types::block::Block;
use starcoin_vm_types::transaction::Transaction;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct BlockFetch {
    block: Block,
}

impl BlockFetch {
    pub fn new_from_file(block_hash: &str) -> BlockFetch {
        let block_hash = HashValue::from_str(block_hash).unwrap();
        BlockFetch {
            block: utils::deserialize_from_file_for_block(block_hash, &block_hash).unwrap(),
        }
    }

    pub fn new_from_remote(block_hash: &str, client: Arc<RpcClient>) -> BlockFetch {
        let block_hash = HashValue::from_str(block_hash).unwrap();

        let block: Block = client
            .chain_get_block_by_hash(block_hash, None)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        utils::serialize_to_file(block.id(), &block.id(), &block).unwrap();
        BlockFetch { block }
    }

    pub fn block_hash(&self) -> HashValue {
        self.block.header.id()
    }

    pub fn parent_block_hash(&self) -> HashValue {
        self.block.header.parent_hash()
    }

    pub fn state_root(&self) -> HashValue {
        self.block.header.state_root()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        types::try_into_transactions(&self.block)
    }
}
