use mock_chain_state::MockChainState;
use mock_reader::MockReader;
use remote_mock_state_node_store::RemoteMockStateNodeStore;
use starcoin_crypto::HashValue;
use starcoin_executor;
use starcoin_rpc_client::RpcClient;
use starcoin_rpc_client::StateRootOption::BlockNumber;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_types::block::Block;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::Arc;

mod mock_chain_state;
mod mock_reader;
mod remote_mock_state_node_store;
mod types;

fn main() {
    let oracle_preimage: HashMap<String, String> = HashMap::new();
    println!("oracle_preimage: {:?}", oracle_preimage);
    let mock_reader = Rc::new(MockReader::new());
    let mock_store = Arc::new(MockStateNodeStore::new());

    let args: Vec<String> = env::args().collect();
    let block_number = args.get(1);
    if block_number.is_some() {
        // ws://main.seed.starcoin.org:9870
        let client = Arc::new(RpcClient::connect_websocket("ws://127.0.0.1:9870").unwrap());
        let block_number: u64 = block_number.unwrap().parse::<u64>().unwrap() as u64;

        // reader build
        let parent_block_number = BlockNumber(block_number - 1);
        let reader = Rc::new(client.state_reader(parent_block_number).unwrap());

        // writer build
        let remote_mock_store = Arc::new(RemoteMockStateNodeStore::new(
            client.clone(),
            mock_store.clone(),
        ));
        let parent_state_root = reader.state_root();
        let writer = ChainStateDB::new(remote_mock_store.clone(), Some(parent_state_root));

        // tx build
        let block = remote_block(block_number, client.clone());

        // mock reader build
        let mock_reader = mock_reader.clone();
        mock_reader.set_parent_state_root(parent_state_root);
        mock_reader.set_block(block);

        let mock_chain_state = MockChainState::new(reader, writer, Some(mock_reader.clone()));
        commit(&mock_chain_state, mock_reader.clone());

        mock_reader.snapshot();
        remote_mock_store.snapshot();
        return;
    } else {
        let writer = ChainStateDB::new(mock_store, Some(mock_reader.parent_state_root()));
        let mock_chain_state = MockChainState::new(mock_reader.clone(), writer, None);
        commit(&mock_chain_state, mock_reader);
    }
}

fn remote_block(block_number: u64, client: Arc<RpcClient>) -> Block {
    client
        .chain_get_block_by_number(block_number, None)
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap()
}

fn commit<S: ChainStateReader + ChainStateWriter>(
    mock_chain_state: &S,
    mock_reader: Rc<MockReader>,
) {
    let executor_data = starcoin_executor::block_execute(
        mock_chain_state,
        mock_reader.transactions(),
        u64::MAX,
        None,
    )
    .unwrap();
    assert_eq!(mock_reader.state_root(), executor_data.state_root);
}
