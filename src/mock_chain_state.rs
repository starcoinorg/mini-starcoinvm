use crate::MockReader;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateWithProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::WriteSet;
use std::rc::Rc;

pub struct MockChainState<R> {
    reader: Rc<R>,
    writer: ChainStateDB,
    mock_reader: Option<Rc<MockReader>>,
}

impl<R> MockChainState<R>
where
    R: ChainStateReader,
{
    pub fn new(
        reader: Rc<R>,
        writer: ChainStateDB,
        mock_reader: Option<Rc<MockReader>>,
    ) -> MockChainState<R> {
        MockChainState {
            reader,
            writer,
            mock_reader,
        }
    }
}

impl<R> ChainStateWriter for MockChainState<R> {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        self.writer.set(access_path, value)
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        self.writer.remove(access_path)
    }

    fn apply(&self, state_set: ChainStateSet) -> Result<()> {
        self.writer.apply(state_set)
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        self.writer.apply_write_set(write_set)
    }

    fn commit(&self) -> Result<HashValue> {
        self.writer.commit()
    }

    fn flush(&self) -> Result<()> {
        self.writer.flush()
    }
}

impl<R> ChainStateReader for MockChainState<R>
where
    R: ChainStateReader,
{
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
        self.writer.state_root()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        unimplemented!()
    }
}

impl<R> StateView for MockChainState<R>
where
    R: ChainStateReader,
{
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let data_path = self.reader.get(access_path)?;
        if let Some(mock) = &self.mock_reader {
            mock.put_data_path(access_path.clone(), data_path.clone());
        }
        Ok(data_path)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
