use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_rpc_client::RpcClient;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use starcoin_state_tree::mock::MockStateNodeStore;
use std::collections::btree_map::BTreeMap;
use std::sync::Arc;

pub struct RemoteMockStateNodeStore {
    client: Arc<RpcClient>,
    store: Arc<MockStateNodeStore>,
}

impl RemoteMockStateNodeStore {
    pub fn new(client: Arc<RpcClient>, store: Arc<MockStateNodeStore>) -> Self {
        Self { client, store }
    }

    pub fn snapshot(&self) {}
}

impl StateNodeStore for RemoteMockStateNodeStore {
    fn get(&self, node_hash: &HashValue) -> Result<Option<StateNode>> {
        let state_node = self.store.get(node_hash)?;
        match state_node {
            Some(state_node) => Ok(Some(state_node)),
            None => {
                let node_hash = node_hash.clone();
                let state_node = self.client.get_state_node_by_node_hash(node_hash)?;
                match state_node {
                    Some(state_node) => {
                        let state_node = StateNode(state_node);
                        self.put(node_hash, state_node.clone()).unwrap();
                        Ok(Some(state_node))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.store.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        self.store.write_nodes(nodes)
    }
}
