pub use crate::file_helper::FileHelper;
use crate::mock_state_view::FileStateView;
#[cfg(any(feature = "from_remote", feature = "test"))]
use crate::mock_state_view::RemoteStateView;
use anyhow::anyhow;
use mock_chain_state::MockChainState;
use mock_state_node_store::MockStateNodeStore;
use starcoin_crypto::HashValue;
#[cfg(any(feature = "from_remote", feature = "test"))]
use starcoin_rpc_client::RpcClient;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_types::block::Block;
use starcoin_types::error::BlockExecutorError;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::transaction::{Transaction, TransactionOutput, TransactionStatus};
use std::sync::Arc;

mod file_helper;
mod mock_chain_state;
mod mock_state_node_store;
mod mock_state_view;
mod types;

#[cfg(any(feature = "from_remote", feature = "test"))]
pub fn remote_state(
    block_hash: HashValue,
    file_helper: Arc<FileHelper>,
    client_ref: &RpcClient,
    client: Arc<RpcClient>,
) -> (HashValue, MockChainState<RemoteStateView>, Vec<Transaction>) {
    let block = block_from_remote(&file_helper, block_hash, block_hash, client_ref);
    let block_parent = block_from_remote(
        &file_helper,
        block_hash,
        block.header.parent_hash(),
        client_ref,
    );

    let state_view = RemoteStateView::new(
        file_helper.clone(),
        client_ref,
        block.header.parent_hash(),
        block.header.id(),
    );
    let state_node_store =
        MockStateNodeStore::new_remote_store(file_helper.clone(), client, block.header.id());
    (
        block.header.state_root(),
        MockChainState::new(state_view.state_root(), state_view, state_node_store),
        types::try_into_transactions(&block_parent, &block),
    )
}

pub fn local_file_state(
    block_hash: HashValue,
    file_helper: Arc<FileHelper>,
) -> (HashValue, MockChainState<FileStateView>, Vec<Transaction>) {
    let block = block_from_file(&file_helper, block_hash, block_hash);
    let block_parent = block_from_file(&file_helper, block_hash, block.header.parent_hash());
    let state_view = FileStateView::new(block.header.id(), file_helper.clone());
    let state_node_store =
        MockStateNodeStore::new_file_store(block.header.id(), file_helper.clone());

    (
        block.header.state_root(),
        MockChainState::new(state_view.state_root(), state_view, state_node_store),
        types::try_into_transactions(&block_parent, &block),
    )
}

fn block_from_file(file_helper: &FileHelper, path: HashValue, block_hash: HashValue) -> Block {
    file_helper
        .deserialize_from_file_for_block(path, &block_hash)
        .unwrap()
}

#[cfg(any(feature = "from_remote", feature = "test"))]
fn block_from_remote(
    file_helper: &FileHelper,
    path: HashValue,
    block_hash: HashValue,
    client: &RpcClient,
) -> Block {
    let block: Block = client
        .chain_get_block_by_hash(block_hash, None)
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    file_helper
        .serialize_to_file(path, &block.id(), &block)
        .unwrap();
    block
}

pub fn block_executor<S: ChainStateReader + ChainStateWriter>(
    chain_state: &S,
    txs: Vec<Transaction>,
) -> anyhow::Result<HashValue> {
    let mut vm = StarcoinVM::new();
    let txn_output: Vec<TransactionOutput> = vm
        .execute_block_transactions(chain_state, txs, Some(u64::MAX))?
        .into_iter()
        .map(|(_, output)| output)
        .collect();

    for output in txn_output {
        let (write_set, _, _, status) = output.into_inner();
        match status {
            TransactionStatus::Discard(status) => {
                return Err(anyhow!("block execution fail, {:?}", status));
            }
            TransactionStatus::Keep(_) => {
                chain_state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;

                chain_state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
            }
        }
    }
    Ok(chain_state.state_root())
}
