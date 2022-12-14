use crate::FileHelper;
use starcoin_crypto::HashValue;
#[cfg(any(feature = "from_remote", feature = "test"))]
use starcoin_rpc_client::{RemoteStateReader, RpcClient, StateRootOption};
use starcoin_state_api::ChainStateReader;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use std::sync::Arc;

const BLOCK_STATE_ROOT: &str = "block_state_root";

#[cfg(any(feature = "from_remote", feature = "test"))]
pub struct RemoteStateView<'a> {
    file_helper: Arc<FileHelper>,
    block_hash_mapping_file: HashValue,
    state_view: RemoteStateReader<'a>,
}

#[cfg(any(feature = "from_remote", feature = "test"))]
impl<'a> RemoteStateView<'a> {
    pub fn new(
        file_helper: Arc<FileHelper>,
        client: &'a RpcClient,
        block_hash: HashValue,
        block_hash_mapping_file: HashValue,
    ) -> RemoteStateView<'a> {
        let state_view = client
            .state_reader(StateRootOption::BlockHash(block_hash))
            .unwrap();
        file_helper
            .serialize_to_file(
                block_hash_mapping_file,
                BLOCK_STATE_ROOT,
                &state_view.state_root(),
            )
            .unwrap();
        RemoteStateView {
            file_helper,
            block_hash_mapping_file,
            state_view,
        }
    }

    pub fn state_root(&self) -> HashValue {
        self.state_view.state_root()
    }
}

#[cfg(any(feature = "from_remote", feature = "test"))]
impl<'a> StateView for RemoteStateView<'a> {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.state_view.get(access_path).map(|op| {
            op.map(|state| {
                self.file_helper
                    .serialize_to_file(self.block_hash_mapping_file, access_path, &state)
                    .unwrap();
                state
            })
        })
    }

    fn is_genesis(&self) -> bool {
        self.state_view.is_genesis()
    }
}

pub struct FileStateView {
    block_hash: HashValue,
    file_helper: Arc<FileHelper>,
}

impl FileStateView {
    pub fn new(block_hash: HashValue, file_helper: Arc<FileHelper>) -> FileStateView {
        FileStateView {
            block_hash,
            file_helper,
        }
    }

    pub fn state_root(&self) -> HashValue {
        self.file_helper
            .deserialize_from_file_for_block_state_root(self.block_hash, BLOCK_STATE_ROOT)
            .unwrap()
    }
}

impl StateView for FileStateView {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.file_helper
            .deserialize_from_file_for_vev_u8(self.block_hash, access_path)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
