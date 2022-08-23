use crate::file_helper;
use anyhow::Result;
use starcoin_crypto::HashValue;
#[cfg(feature = "from_remote")]
use starcoin_rpc_client::RpcClient;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) struct MockStateNodeStore {
    store: starcoin_state_tree::mock::MockStateNodeStore,
    handler: Box<dyn Fn(HashValue) -> Result<Option<Vec<u8>>> + Send + Sync>,
}

impl MockStateNodeStore {
    #[cfg(feature = "from_file")]
    pub fn new_file_store(block_hash: HashValue) -> impl StateNodeStore {
        MockStateNodeStore {
            store: starcoin_state_tree::mock::MockStateNodeStore::new(),
            handler: Box::new(move |node_hash: HashValue| {
                file_helper::deserialize_from_file_for_vev_u8(block_hash, &node_hash)
            }),
        }
    }

    #[cfg(feature = "from_remote")]
    pub fn new_remote_store(
        client: Arc<RpcClient>,
        block_hash_mapping_file: HashValue,
    ) -> impl StateNodeStore {
        MockStateNodeStore {
            store: starcoin_state_tree::mock::MockStateNodeStore::new(),
            handler: Box::new(move |node_hash: HashValue| {
                client
                    .get_state_node_by_node_hash(node_hash)
                    .map(|op| match op {
                        None => {
                            file_helper::skip_step();
                            None
                        }
                        Some(state_node) => {
                            file_helper::serialize_to_file(
                                block_hash_mapping_file,
                                &node_hash,
                                &state_node,
                            )
                            .unwrap();
                            Some(state_node)
                        }
                    })
            }),
        }
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, node_hash: &HashValue) -> anyhow::Result<Option<StateNode>> {
        let state_node = self.store.get(node_hash)?;
        match state_node {
            Some(state_node) => Ok(Some(state_node)),
            None => {
                let node_hash = node_hash.clone();
                match (self.handler)(node_hash)? {
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

    fn put(&self, key: HashValue, node: StateNode) -> anyhow::Result<()> {
        self.store.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> anyhow::Result<()> {
        self.store.write_nodes(nodes)
    }
}
