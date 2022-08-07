use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_executor;
use starcoin_network_rpc_api::RemoteChainStateReader;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockTransactionsView, BlockTransactionsView::Full, BlockView,
    DecodedScriptFunctionView, RawUserTransactionView, TransactionPayloadView,
    TransactionPayloadView::*,
};
use starcoin_rpc_client::StateRootOption;
use starcoin_rpc_client::StateRootOption::BlockNumber;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateWithProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::{Block, BlockBody, BlockHeader};
use starcoin_types::error::ExecutorResult;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{
    Package, RawUserTransaction, Script, ScriptFunction, SignedUserTransaction, Transaction,
    TransactionPayload,
};
use starcoin_vm_types::write_set::WriteSet;
use std::hash::Hash;

fn main() {
    let url = "ws://main.seed.starcoin.org:9870";
    let client = RpcClient::connect_websocket(url).unwrap();
    let block_number = 7164470u64;

    let block_view = client
        .chain_get_block_by_number(block_number, None)
        .unwrap()
        .unwrap();
    println!("{:?}", block_view);

    // tx build
    let parent_gas_used = 10000u64;
    let block = try_from_block_view(&block_view).unwrap();
    let transactions = try_into_transactions(&block, Some(parent_gas_used));

    // chain_state build
    let block_number = BlockNumber(block_number);
    let reader = client
        .state_reader(block_number)
        .expect("TODO: panic message");
    println!("stateRoot: {:?}", reader.state_root());

    let writer = ChainStateDB::mock();

    let block_gas_limit = 10000u64;
    let mock_chain_state = MockChainStateDB::new(&reader, &writer);

    // SEQUENCE_NUMBER_TOO_OLD
    // INVALID_TIMESTAMP -> EINVALID_TIMESTAMP
    let executor_data =
        starcoin_executor::block_execute(&mock_chain_state, transactions, block_gas_limit, None);
    println!("next stateRoot: {:?}", executor_data)
}

fn try_from_block_view(block_view: &BlockView) -> Result<Block, Error> {
    let block_header = from_block_header_view(&block_view.header);
    let uncles: Vec<BlockHeader> = block_view
        .uncles
        .iter()
        .map(|uncle| from_block_header_view(&uncle))
        .collect();
    let transactions = from_block_view_body(&block_view.body).unwrap();

    Ok(Block {
        header: block_header,
        body: BlockBody::new(transactions, Some(uncles)),
    })
}

fn from_block_view_body(body_view: &BlockTransactionsView) -> Option<Vec<SignedUserTransaction>> {
    let transactions: Option<Vec<SignedUserTransaction>> =
        if let Full(transactions_view) = body_view {
            Some(
                transactions_view
                    .iter()
                    .map(|transaction| {
                        SignedUserTransaction::new(
                            from_transaction_view(&transaction.raw_txn),
                            transaction.authenticator.clone(),
                        )
                    })
                    .collect(),
            )
        } else {
            None
        };
    transactions
}

fn from_transaction_view(transaction_view: &RawUserTransactionView) -> RawUserTransaction {
    RawUserTransaction::new(
        transaction_view.sender,
        transaction_view.sequence_number.0,
        TransactionPayload::decode(transaction_view.payload.0.as_slice()).unwrap(),
        transaction_view.max_gas_amount.0,
        transaction_view.gas_unit_price.0,
        transaction_view.expiration_timestamp_secs.0,
        ChainId::new(transaction_view.chain_id),
        transaction_view.gas_token_code.clone(),
    )
}

// fn from_transaction_payload_view(
//     transaction_payload_view: TransactionPayloadView,
// ) -> TransactionPayload {
//     match transaction_payload_view {
//         Script(script_view) => TransactionPayload::Script(Script::new(
//             script_view.code.0,
//             script_view.ty_args.iter().map(|ty_arg| ty_arg.0).collect(),
//             script_view.args.0,
//         )),
//         Package(package_view) => TransactionPayload::Package(
//             Package::new(
//                 package_view.modules.iter().map(|module| module.0).collect(),
//                 from_decoded_script_function_view(package_view.init_script.unwrap()),
//             )
//             .unwrap(),
//         ),
//         ScriptFunction(script_function_view) => {
//             TransactionPayload::ScriptFunction(ScriptFunction::new((), (), vec![], vec![]))
//         }
//     }
// }
//
// fn from_decoded_script_function_view(
//     decode_script_function_view: DecodedScriptFunctionView,
// ) -> Option<ScriptFunction> {
//     Some(ScriptFunction::new(
//         decode_script_function_view.module.0,
//         decode_script_function_view.function,
//         decode_script_function_view
//             .ty_args
//             .iter()
//             .map(|type_tag_view| type_tag_view.0)
//             .collect(),
//         vec![],
//     ))
// }

fn from_block_header_view(header_view: &BlockHeaderView) -> BlockHeader {
    BlockHeader::new(
        header_view.parent_hash,
        header_view.timestamp.0,
        header_view.number.0,
        header_view.author,
        header_view.txn_accumulator_root,
        header_view.block_accumulator_root,
        header_view.state_root,
        header_view.gas_used.0,
        header_view.difficulty,
        header_view.body_hash,
        ChainId::new(header_view.chain_id),
        header_view.nonce,
        header_view.extra,
    )
}

fn try_into_transactions(block: &Block, parent_gas_used: Option<u64>) -> Vec<Transaction> {
    let mut t = match &parent_gas_used {
        None => vec![],
        Some(gas_used) => {
            let block_metadata = block.to_metadata(*gas_used);
            vec![Transaction::BlockMetadata(block_metadata)]
        }
    };
    t.extend(
        block
            .transactions()
            .iter()
            .cloned()
            .map(Transaction::UserTransaction),
    );
    t
}

struct MockChainStateDB<'a> {
    writer: &'a dyn ChainStateWriter,
    reader: &'a dyn ChainStateReader,
}

impl<'a> MockChainStateDB<'a> {
    pub fn new(
        reader: &'a dyn ChainStateReader,
        writer: &'a dyn ChainStateWriter,
    ) -> MockChainStateDB<'a> {
        MockChainStateDB { writer, reader }
    }
}

impl<'a> ChainStateWriter for MockChainStateDB<'a> {
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

impl<'a> ChainStateReader for MockChainStateDB<'a> {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.reader.get_with_proof(access_path)
    }

    fn get_account_state(&self, _: &AccountAddress) -> Result<Option<AccountState>> {
        panic!("mock no support get_account_state")
    }

    fn get_account_state_set(&self, _: &AccountAddress) -> Result<Option<AccountStateSet>> {
        panic!("mock no support get_account_state_set")
    }

    fn state_root(&self) -> HashValue {
        self.reader.state_root()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        panic!("mock no support dump")
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        panic!("mock no support dump_iter")
    }
}

impl<'a> StateView for MockChainStateDB<'a> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.reader.get(access_path)
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
