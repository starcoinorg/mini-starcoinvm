use clap::Parser;
#[cfg(feature = "from_remote")]
use mini_starcoin_vm::remote_state;
#[cfg(feature = "from_remote")]
use starcoin_rpc_client::RpcClient;

use mini_starcoin_vm::{block_executor, FileHelper};
use starcoin_crypto::HashValue;
use std::sync::Arc;

#[derive(Parser)]
struct Options {
    #[clap(short = 'b', long = "block")]
    block_hash: HashValue,
    #[cfg(feature = "from_remote")]
    #[clap(short = 'n', long = "network")]
    /// Maybe: main, halley, local, barnard
    chain_network: String,
    #[clap(short = 'p', long = "prefix", default_value = ".")]
    prefix_path: String,
}

fn main() {
    let opts: Options = Options::parse();
    let block_hash = opts.block_hash;
    let file_helper = Arc::new(FileHelper::new(opts.prefix_path));

    #[cfg(feature = "from_remote")]
    let client = {
        let chain_network = match opts.chain_network.to_lowercase().as_str() {
            "main" => "ws://main.seed.starcoin.org:9870",
            "halley" => "ws://halley.seed.starcoin.org:9870",
            "local" => "ws://192.168.1.101:9870",
            "barnard" => "ws://barnard.seed.starcoin.org:9870",
            _ => panic!("network not support yet"),
        };
        Arc::new(RpcClient::connect_websocket(chain_network).unwrap())
    };
    #[cfg(feature = "from_remote")]
    let (block_state_root, mock_chain_state, txs) = {
        file_helper.init_file_path(block_hash).unwrap();
        remote_state(block_hash, file_helper, &client, client.clone())
    };
    #[cfg(not(feature = "from_remote"))]
    let (block_state_root, mock_chain_state, txs) =
        mini_starcoin_vm::local_file_state(block_hash, file_helper);
    let execution_state_root = block_executor(&mock_chain_state, txs).unwrap();
    assert_eq!(block_state_root, execution_state_root);
}
