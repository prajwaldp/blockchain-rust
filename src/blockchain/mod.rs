use std::collections::HashMap;

pub mod block;
pub mod transaction;

use block::Block;
use transaction::{Transaction, TxnOutput};

#[derive(Debug)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
    pub last_hash: Vec<u8>,
}

impl BlockChain {
    pub fn new(address: &'static str) -> Self {
        let coinbase_txn = Transaction::create_coinbase_txn(address, "First Transaction");
        let genesis_block = Block::create_genesis_block(coinbase_txn);

        BlockChain {
            blocks: vec![genesis_block.clone()],
            last_hash: genesis_block.hash,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        let last_hash = block.hash.clone();
        self.blocks.push(block);
        self.last_hash = last_hash;
    }

    pub fn add_transaction(&mut self, from: &'static str, to: &'static str, amount: i32) {
        let txn = Transaction::new(from, to, amount, &self);
        let block = Block::create(vec![txn], self.last_hash.clone());
        self.add_block(block);
    }

    pub fn find_unspent_transactions(&self, address: &'static str) -> Vec<&Transaction> {
        let mut unspent_transactions: Vec<&Transaction> = Vec::new();
        let mut spent_txos = HashMap::<String, Vec<i32>>::new();

        for block in self.blocks.iter().rev() {
            for txn in &block.transactions {
                let txn_id = hex::encode(&txn.id);

                for (out_idx, out) in txn.outputs.iter().enumerate() {
                    let mut found: bool = false;
                    if spent_txos.contains_key(&txn_id) {
                        for spent_output in &spent_txos[&txn_id] {
                            if *spent_output == out_idx as i32 {
                                found = true;
                                break;
                            }
                        }
                    }

                    if found {
                        continue;
                    }

                    if out.can_be_unlocked(address) {
                        unspent_transactions.push(&txn);
                    }
                }

                if !txn.is_coinbase() {
                    for input in &txn.inputs {
                        if input.can_unlock(address) {
                            let txn_id = hex::encode(&input.id);
                            spent_txos.entry(txn_id).or_insert(vec![]).push(input.out);
                        }
                    }
                }
            }
        }

        unspent_transactions
    }

    pub fn find_unspent_txn_outputs(&self, address: &'static str) -> Vec<&TxnOutput> {
        let mut unspent_txn_outputs: Vec<&TxnOutput> = Vec::new();
        let unspent_txns = self.find_unspent_transactions(address);

        for txn in &unspent_txns {
            for output in &txn.outputs {
                if output.can_be_unlocked(address) {
                    unspent_txn_outputs.push(output);
                }
            }
        }

        unspent_txn_outputs
    }

    pub fn find_spendable_outputs(
        &self,
        address: &'static str,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs = HashMap::<String, Vec<i32>>::new();
        let unspent_txns = self.find_unspent_transactions(address);
        let mut accumulated: i32 = 0;

        let mut txn_id;
        for txn in unspent_txns {
            txn_id = hex::encode(&txn.id).to_owned();

            for (output_idx, output) in txn.outputs.iter().enumerate() {
                if output.can_be_unlocked(address) && accumulated < amount {
                    accumulated += output.value;
                    unspent_outputs
                        .entry(txn_id.clone())
                        .or_insert(vec![])
                        .push(output_idx as i32);
                }

                if accumulated >= amount {
                    return (accumulated, unspent_outputs);
                }
            }
        }

        (accumulated, unspent_outputs)
    }
}

impl std::fmt::Display for BlockChain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Blockchain
Last Hash: {}
Blocks: {}",
            hex::encode(&self.last_hash),
            self.blocks
                .iter()
                .map(|b| format!("{}", b))
                .collect::<String>()
        )
    }
}
