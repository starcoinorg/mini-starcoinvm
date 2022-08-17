use crate::types;
use anyhow::anyhow;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateWithProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::Block;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::Transaction;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub struct MockReader {
    transactions: RefCell<Vec<Transaction>>,
    state_root: Cell<HashValue>,
    parent_state_root: Cell<HashValue>,
    data_path_map: RefCell<HashMap<AccessPath, Option<Vec<u8>>>>,
}

impl MockReader {
    pub fn new() -> Self {
        MockReader {
            state_root: Default::default(),
            transactions: Default::default(),
            parent_state_root: Default::default(),
            data_path_map: Default::default(),
        }
    }

    pub fn put_data_path(&self, access_path: AccessPath, data_path: Option<Vec<u8>>) {
        let mut data_path_map = self.data_path_map.borrow_mut();
        data_path_map.insert(access_path, data_path);
    }

    pub fn set_parent_state_root(&self, parent_state_root: HashValue) {
        self.parent_state_root.set(parent_state_root)
    }

    pub fn set_block(&self, block: Block) {
        self.state_root.set(block.header.state_root());
        let mut transactions = self.transactions.borrow_mut();
        transactions.extend(types::try_into_transactions(&block));
    }

    pub fn state_root(&self) -> HashValue {
        self.state_root.get()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.transactions.borrow().clone()
    }

    pub fn parent_state_root(&self) -> HashValue {
        self.parent_state_root.get()
    }

    pub fn snapshot(&self) {}
}

impl StateView for MockReader {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.data_path_map.borrow().get(access_path) {
            Some(data_path) => Ok(data_path.clone()),
            None => Err(anyhow!(format!("access path missing: {}", access_path))),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

impl ChainStateReader for MockReader {
    fn get_with_proof(&self, _: &AccessPath) -> anyhow::Result<StateWithProof> {
        unimplemented!()
    }

    fn get_account_state(&self, _: &AccountAddress) -> anyhow::Result<Option<AccountState>> {
        unimplemented!()
    }

    fn get_account_state_set(&self, _: &AccountAddress) -> anyhow::Result<Option<AccountStateSet>> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        self.parent_state_root.get()
    }

    fn dump(&self) -> anyhow::Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> anyhow::Result<AccountStateSetIterator> {
        unimplemented!()
    }
}
