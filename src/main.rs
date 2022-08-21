use crate::mock_state_view::{FileStateView, RemoteStateView};
use block_fetch::BlockFetch;
use mock_chain_state::MockChainState;
use mock_state_node_store::MockStateNodeStore;
use starcoin_crypto::HashValue;
use starcoin_executor;
use starcoin_rpc_client::RpcClient;
use std::borrow::Borrow;
use std::str::FromStr;
use std::sync::Arc;

mod block_fetch;
mod mock_chain_state;
mod mock_state_node_store;
mod mock_state_view;
mod types;
mod utils;

fn main() {
    let config_client_url = "ws://192.168.1.101:9870";
    let block_hash = "0xd84e80411d3cbaf09a4c2eeebaa941f351191292d16ec92965368863e636f27c";

    let block;
    let mock_chain_state;

    #[cfg(target_arch = "x86_64")]
    let client;
    #[cfg(target_arch = "x86_64")]
    {
        utils::init_file_path(HashValue::from_str(block_hash).unwrap()).unwrap();
        client = Arc::new(RpcClient::connect_websocket(config_client_url).unwrap());
        block = BlockFetch::new_from_remote(block_hash, client.clone());
        let state_view = RemoteStateView::new(
            client.borrow(),
            block.parent_block_hash(),
            block.block_hash(),
        );
        mock_chain_state = MockChainState::new(
            state_view.state_root(),
            state_view,
            MockStateNodeStore::new_remote_store(client.clone(), block.block_hash()),
        );
    }

    #[cfg(target_arch = "mips")]
    {
        block = BlockFetch::new_from_file(block_hash);
        let state_view = FileStateView::new(block.block_hash());
        mock_chain_state = MockChainState::new(
            state_view.state_root(),
            state_view,
            MockStateNodeStore::new_file_store(block.block_hash()),
        );
    }

    let executor_data =
        starcoin_executor::block_execute(&mock_chain_state, block.transactions(), u64::MAX, None)
            .unwrap();
    // print!("block: {:?}, executor_data: {:?}", block, executor_data);
    assert_eq!(block.state_root(), executor_data.state_root);
}
