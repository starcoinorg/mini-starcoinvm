use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateWithProof};
use starcoin_state_store_api::StateNodeStore;
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::WriteSet;
use std::sync::Arc;

pub struct MockChainState<S> {
    state_view: S,
    chain_state_db: ChainStateDB,
}

impl<S: StateView> MockChainState<S> {
    pub fn new(
        state_root: HashValue,
        state_view: S,
        store: impl StateNodeStore + 'static,
    ) -> MockChainState<S> {
        MockChainState {
            state_view,
            chain_state_db: ChainStateDB::new(Arc::new(store), Some(state_root)),
        }
    }
}

impl<S> ChainStateWriter for MockChainState<S> {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        self.chain_state_db.set(access_path, value)
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        self.chain_state_db.remove(access_path)
    }

    fn apply(&self, state_set: ChainStateSet) -> Result<()> {
        self.chain_state_db.apply(state_set)
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        self.chain_state_db.apply_write_set(write_set)
    }

    fn commit(&self) -> Result<HashValue> {
        self.chain_state_db.commit()
    }

    fn flush(&self) -> Result<()> {
        self.chain_state_db.flush()
    }
}

impl<S: StateView> ChainStateReader for MockChainState<S> {
    fn get_with_proof(&self, _: &AccessPath) -> Result<StateWithProof> {
        unimplemented!()
    }

    fn get_account_state(&self, _: &AccountAddress) -> Result<Option<AccountState>> {
        unimplemented!()
    }

    fn get_account_state_set(&self, _: &AccountAddress) -> Result<Option<AccountStateSet>> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        self.chain_state_db.state_root()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        unimplemented!()
    }
}

impl<S: StateView> StateView for MockChainState<S> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self.state_view.get(access_path)?)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
