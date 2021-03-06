use std::collections::HashMap;

pub mod block;
pub mod merkle;
pub mod transaction;
pub mod txn;
pub mod wallet;

use crate::util::constants::BLOCK_MEMORY_POOL_SIZE;
use crate::util::types::Bytes;
use block::Block;
use log::{info, warn};
use serde::Serialize;
use transaction::Transaction;
use txn::TxnOutput;

#[derive(Clone, Debug, Default, Serialize)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
    pub last_hash: Vec<u8>,
    pub length: i32,
    memory_pool: Vec<Vec<Block>>,
}

impl BlockChain {
    pub fn new(address: &Bytes) -> Self {
        let coinbase_txn = Transaction::create_coinbase_txn(address);
        let genesis_block = Block::create_genesis_block(coinbase_txn);

        BlockChain {
            blocks: vec![genesis_block.clone()],
            last_hash: genesis_block.hash,
            length: 1,
            memory_pool: vec![],
        }
    }

    pub fn new_placeholder() -> Self {
        BlockChain {
            blocks: vec![],
            last_hash: vec![],
            length: 0,
            memory_pool: vec![],
        }
    }

    pub fn add_block(&mut self, block: Block) {
        let last_hash = block.hash.clone();
        self.blocks.push(block);
        self.last_hash = last_hash;
        self.length += 1;
    }

    pub fn add_block_to_memory_pool(&mut self, block: Block) {
        if self.memory_pool.is_empty() {
            info!("[Blockchain] Memory Pool is empty");

            if block.index == self.length
                && block.timestamp >= self.blocks.last().unwrap().timestamp
                && block.prev_hash == self.last_hash
            {
                info!(
                    "[Blockchain] Added block {} to a new candidate in the memory pool",
                    hex::encode(&block.hash)
                );
                self.memory_pool.push(vec![block]);
            } else {
                warn!(
                    "[Blockchain] Cannot add the block {} (index: {}, timestamp: {}) to blockchain with last hash {} and length {}",
                    hex::encode(&block.hash),
                    &block.index,
                    &block.timestamp,
                    hex::encode(&self.last_hash),
                    &self.length
                );
            }
        } else {
            // 1. Push the block to all inner Vectors with matching hash, index
            //    and timestamp
            // 2. Keep track of the length

            for candidate in self.memory_pool.iter_mut() {
                let last_block = candidate.last().unwrap();
                if block.index == last_block.index + 1
                    && block.timestamp >= last_block.timestamp
                    && block.prev_hash == last_block.hash
                {
                    info!(
                        "[Blockchain] Added block {} to an existing candidate in the memory pool",
                        hex::encode(&block.hash)
                    );
                    candidate.push(block.clone());
                }
            }
        }

        self.clear_memory_pool();
    }

    pub fn clear_memory_pool(&mut self) {
        let mut selected_candidate = Vec::<Block>::new();
        let mut candidate_found: bool = false;

        for candidate in self.memory_pool.iter_mut() {
            if candidate.len() == BLOCK_MEMORY_POOL_SIZE {
                selected_candidate = candidate.clone();
                candidate_found = true;
                info!("[Blockchain] Memory Pool Capacity Reached");
                break;
            }
        }

        for block in selected_candidate.iter() {
            info!(
                "[Blockchain] Adding block {} to blockchain",
                hex::encode(&block.hash)
            );
            self.add_block(block.clone());
        }

        if candidate_found {
            self.memory_pool = vec![];
        }
    }

    pub fn find_unspent_transactions(&self, public_key_hash: &Bytes) -> Vec<&Transaction> {
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

                    if out.is_locked_with_key(&public_key_hash) {
                        unspent_transactions.push(&txn);
                    }
                }

                if !txn.is_coinbase() {
                    for input in &txn.inputs {
                        if input.is_uses_key(&public_key_hash) {
                            let txn_id = hex::encode(&input.id);
                            spent_txos.entry(txn_id).or_insert(vec![]).push(input.out);
                        }
                    }
                }
            }
        }

        unspent_transactions
    }

    #[allow(dead_code)]
    pub fn find_unspent_txn_outputs(&self) -> HashMap<String, Vec<&TxnOutput>> {
        let mut unspent_txn_outputs = HashMap::<String, Vec<&TxnOutput>>::new();
        let mut spent_txn_outputs = HashMap::<String, Vec<i32>>::new();

        for block in self.blocks.iter().rev() {
            for txn in block.transactions.iter() {
                let txn_id = hex::encode(&txn.id);
                let is_txn_id_in_spent_txn_outputs = spent_txn_outputs.contains_key(&txn_id);
                let mut found: bool = false;

                for (output_idx, output) in txn.outputs.iter().enumerate() {
                    if is_txn_id_in_spent_txn_outputs {
                        for spent_output in spent_txn_outputs[&txn_id].iter() {
                            if *spent_output == output_idx as i32 {
                                found = true;
                                break;
                            }
                        }

                        if found {
                            continue;
                        }

                        let mut outputs = unspent_txn_outputs[&txn_id].clone();
                        outputs.push(output);
                        unspent_txn_outputs
                            .entry(txn_id.clone())
                            .or_insert(vec![])
                            .append(&mut outputs);
                    }

                    if !txn.is_coinbase() {
                        for input in txn.inputs.iter() {
                            let input_txn_id = hex::encode(&input.id);
                            spent_txn_outputs
                                .entry(input_txn_id)
                                .or_insert(vec![])
                                .push(input.out);
                        }
                    }
                }
            }
        }

        unspent_txn_outputs
    }

    pub fn find_spendable_outputs(
        &self,
        public_key_hash: &Bytes,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs = HashMap::<String, Vec<i32>>::new();
        let unspent_txns = self.find_unspent_transactions(&public_key_hash);
        let mut accumulated: i32 = 0;

        let mut txn_id;
        for txn in unspent_txns {
            txn_id = hex::encode(&txn.id).to_owned();

            for (output_idx, output) in txn.outputs.iter().enumerate() {
                if output.is_locked_with_key(&public_key_hash) && accumulated < amount {
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

    pub fn find_transaction(&self, id: &Bytes) -> Result<&Transaction, &str> {
        for block in self.blocks.iter().rev() {
            for txn in block.transactions.iter() {
                if txn.id == *id {
                    return Ok(txn);
                }
            }

            if block.prev_hash.len() == 0 {
                break;
            }
        }

        Err("Transaction not found")
    }

    pub fn sign_transaction(&self, txn: &mut Transaction, private_key: secp256k1::SecretKey) {
        let mut prev_txns = HashMap::<String, &Transaction>::new();

        for txn_input in txn.inputs.iter() {
            let prev_txn = self.find_transaction(&txn_input.id).unwrap();
            prev_txns.insert(hex::encode(&prev_txn.id), prev_txn);
        }

        txn.sign(private_key, prev_txns).expect("");
    }

    #[allow(dead_code)]
    pub fn verify_transaction(&self, txn: &mut Transaction) -> bool {
        let mut prev_txns = HashMap::<String, &Transaction>::new();

        for txn_input in txn.inputs.iter() {
            let prev_txn = self.find_transaction(&txn_input.id).unwrap();
            prev_txns.insert(hex::encode(&prev_txn.id), prev_txn);
        }

        txn.verify(prev_txns).unwrap()
    }
}

impl std::fmt::Display for BlockChain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
Blockchain
Last Hash: {}
Length: {}
Blocks: {}",
            hex::encode(&self.last_hash),
            self.length,
            self.blocks
                .iter()
                .map(|b| format!("{}", b))
                .collect::<String>()
        )
    }
}
