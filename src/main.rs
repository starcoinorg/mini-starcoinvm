use clap::Parser;
use mini_starcoin_vm::local_file_state;
#[cfg(feature = "from_remote")]
use mini_starcoin_vm::remote_state;
use mini_starcoin_vm::{block_executor, FileHelper};
use starcoin_crypto::HashValue;
#[cfg(feature = "from_remote")]
use starcoin_rpc_client::RpcClient;
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

    let block_state_root;
    let mock_chain_state;
    let txs;

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

        file_helper.init_file_path(block_hash).unwrap();
        client = Arc::new(RpcClient::connect_websocket(chain_network).unwrap());
        (block_state_root, mock_chain_state, txs) =
            remote_state(block_hash, file_helper, &client, client.clone());
    }

    #[cfg(feature = "from_file")]
    {
        (block_state_root, mock_chain_state, txs) = local_file_state(block_hash, file_helper);
    }

    let execution_state_root = block_executor(&mock_chain_state, txs).unwrap();
    assert_eq!(block_state_root, execution_state_root);
}
