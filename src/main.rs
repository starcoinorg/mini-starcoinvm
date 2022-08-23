#[cfg(feature = "from_file")]
use crate::mock_state_view::FileStateView;
#[cfg(feature = "from_remote")]
use crate::mock_state_view::RemoteStateView;
use clap::Parser;
use mock_chain_state::MockChainState;
use mock_state_node_store::MockStateNodeStore;
use starcoin_crypto::HashValue;
use starcoin_executor;
#[cfg(feature = "from_remote")]
use starcoin_rpc_client::RpcClient;
use starcoin_types::block::Block;
#[cfg(feature = "from_remote")]
use std::borrow::Borrow;
#[cfg(feature = "from_remote")]
use std::sync::Arc;

mod file_helper;
mod mock_chain_state;
mod mock_state_node_store;
mod mock_state_view;
mod types;

#[derive(Parser)]
struct Options {
    #[clap(short = 'b', long = "block")]
    block_hash: HashValue,
    #[cfg(feature = "from_remote")]
    #[clap(short = 'n', long = "network")]
    /// Maybe: main, halley, local, barnard
    chain_network: String,
}

fn main() {
    let opts: Options = Options::parse();
    let block_hash = opts.block_hash;

    let block;
    let mock_chain_state;

    #[cfg(feature = "from_remote")]
    let client;
    #[cfg(feature = "from_remote")]
    {
        let chain_network = match opts.chain_network.to_lowercase().as_str() {
            "main" => "ws://main.seed.starcoin.org:9870",
            "halley" => "ws://halley.seed.starcoin.org:9870",
            "local" => "ws://192.168.1.101:9870",
            "barnard" => "ws://barnard.seed.starcoin.org:9870",
            _ => panic!("network not support yet"),
        };

        file_helper::init_file_path(block_hash).unwrap();
        client = Arc::new(RpcClient::connect_websocket(chain_network).unwrap());
        block = new_from_remote(block_hash, client.clone());
        let state_view = RemoteStateView::new(
            client.borrow(),
            block.header.parent_hash(),
            block.header.id(),
        );
        mock_chain_state = MockChainState::new(
            state_view.state_root(),
            state_view,
            MockStateNodeStore::new_remote_store(client.clone(), block.header.id()),
        );
    }

    #[cfg(feature = "from_file")]
    {
        block = new_from_file(block_hash);
        let state_view = FileStateView::new(block.header.id());
        mock_chain_state = MockChainState::new(
            state_view.state_root(),
            state_view,
            MockStateNodeStore::new_file_store(block.header.id()),
        );
    }

    let executor_data = starcoin_executor::block_execute(
        &mock_chain_state,
        types::try_into_transactions(&block),
        u64::MAX,
        None,
    )
    .unwrap();
    assert_eq!(block.header.state_root(), executor_data.state_root);
}

#[cfg(feature = "from_file")]
pub fn new_from_file(block_hash: HashValue) -> Block {
    file_helper::deserialize_from_file_for_block(block_hash, &block_hash).unwrap()
}

#[cfg(feature = "from_remote")]
pub fn new_from_remote(block_hash: HashValue, client: Arc<RpcClient>) -> Block {
    let block: Block = client
        .chain_get_block_by_hash(block_hash, None)
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    file_helper::serialize_to_file(block.id(), &block.id(), &block).unwrap();
    block
}
