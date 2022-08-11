use starcoin_types::block::Block;
use starcoin_vm_types::transaction::Transaction;

pub fn try_into_transactions(block: &Block) -> Vec<Transaction> {
    let mut t = vec![];
    t.push(Transaction::BlockMetadata(
        block.to_metadata(block.header.gas_used()),
    ));
    t.extend(
        block
            .transactions()
            .iter()
            .cloned()
            .map(Transaction::UserTransaction),
    );
    t
}
