use crate::mock_state_view::{FileStateView, RemoteStateView};
use block_fetch::BlockFetch;
use clap::Parser;
use mock_chain_state::MockChainState;
use mock_state_node_store::MockStateNodeStore;
use starcoin_crypto::HashValue;
use starcoin_executor;
use starcoin_rpc_client::RpcClient;
use std::borrow::Borrow;
use std::str::FromStr;
use std::sync::Arc;

mod block_fetch;
mod file_helper;
mod mock_chain_state;
mod mock_state_node_store;
mod mock_state_view;
mod types;

#[derive(Parser)]
struct Options {
    #[clap(short = 'b', long = "block")]
    block_hash: HashValue,
    #[cfg(target_arch = "x86_64")]
    #[clap(short = 'n', long = "network")]
    /// Maybe: main, halley, local, barnard
    chain_network: String,
}

fn main() {
    let opts: Options = Options::parse();
    let block_hash = opts.block_hash;

    let block;
    let mock_chain_state;

    #[cfg(target_arch = "x86_64")]
    let client;
    #[cfg(target_arch = "x86_64")]
    {
        let chain_network = match opts.chain_network.to_lowercase().as_str() {
            "main" => "ws://main1.seed.starcoin.org:9101",
            "halley" => "ws://halley1.seed.starcoin.org:9101",
            "local" => "ws://192.168.1.101:9870",
            "barnard" => "ws://barnard1.seed.starcoin.org:9101",
            _ => panic!("network not support yet"),
        };

        file_helper::init_file_path(block_hash).unwrap();
        client = Arc::new(RpcClient::connect_websocket(chain_network).unwrap());
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
    assert_eq!(block.state_root(), executor_data.state_root);
}
