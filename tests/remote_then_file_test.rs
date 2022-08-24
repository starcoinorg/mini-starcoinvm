#[cfg(test)]
mod tests {
    use mini_starcoin_vm::{block_executor, local_file_state, remote_state, FileHelper};
    use starcoin_crypto::HashValue;
    use starcoin_rpc_client::RpcClient;
    use std::str::FromStr;
    use std::sync::Arc;

    #[test]
    fn test() {
        let halley = "ws://halley.seed.starcoin.org:9870";
        let client = Arc::new(RpcClient::connect_websocket(halley).unwrap());
        let file_helper = Arc::new(FileHelper::new("./remote_then_file_test".to_string()));
        let block_hash = HashValue::from_str(
            "0xe7b7a3309ac1464f655066ba861702a609a8586c5b5f3b2ca9dd15d889d6f5ca",
        )
        .unwrap();

        file_helper.init_file_path(block_hash).unwrap();
        let (block_state_root, mock_chain_state, txs) =
            remote_state(block_hash, file_helper.clone(), &client, client.clone());
        let remote_block_state_root = block_executor(&mock_chain_state, txs).unwrap();
        assert_eq!(remote_block_state_root, block_state_root);

        let (block_state_root, mock_chain_state, txs) = local_file_state(block_hash, file_helper);
        let local_block_state_root = block_executor(&mock_chain_state, txs).unwrap();
        assert_eq!(local_block_state_root, block_state_root);

        assert_eq!(remote_block_state_root, local_block_state_root);
    }
}
