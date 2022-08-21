use crate::utils;
use starcoin_crypto::HashValue;
use starcoin_rpc_client::{RemoteStateReader, RpcClient, StateRootOption};
use starcoin_state_api::ChainStateReader;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use std::str::FromStr;
use std::sync::Arc;

const BLOCK_STATE_ROOT: &str = "block_state_root";

pub(crate) struct RemoteStateView<'a> {
    block_hash: HashValue,
    block_hash_mapping_file: HashValue,
    state_view: RemoteStateReader<'a>,
}

impl<'a> RemoteStateView<'a> {
    pub fn new(
        client: &'a RpcClient,
        block_hash: HashValue,
        block_hash_mapping_file: HashValue,
    ) -> RemoteStateView<'a> {
        let state_view = client
            .state_reader(StateRootOption::BlockHash(block_hash))
            .unwrap();
        utils::serialize_to_file(
            block_hash_mapping_file,
            BLOCK_STATE_ROOT,
            &state_view.state_root(),
        )
        .unwrap();
        RemoteStateView {
            block_hash,
            block_hash_mapping_file,
            state_view,
        }
    }

    pub fn state_root(&self) -> HashValue {
        self.state_view.state_root()
    }
}

impl<'a> StateView for RemoteStateView<'a> {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.state_view.get(access_path).map(|op| {
            op.map(|state| {
                utils::serialize_to_file(self.block_hash_mapping_file, access_path, &state)
                    .unwrap();
                state
            })
        })
    }

    fn is_genesis(&self) -> bool {
        self.state_view.is_genesis()
    }
}

pub(crate) struct FileStateView(HashValue);

impl FileStateView {
    pub fn new(block_hash: HashValue) -> FileStateView {
        FileStateView(block_hash)
    }

    pub fn state_root(&self) -> HashValue {
        utils::deserialize_from_file_for_block_state_root(self.0, BLOCK_STATE_ROOT).unwrap()
    }
}

impl StateView for FileStateView {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(Some(
            utils::deserialize_from_file_for_access_path(self.0, access_path).unwrap(),
        ))
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
