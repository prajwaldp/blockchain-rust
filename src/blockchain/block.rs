use crate::blockchain;
use crate::blockchain::transaction::Transaction;
use crate::util::constants::DIFFICULTY;
use crate::util::traits::Hashable;
use crate::util::types::Bytes;

use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct Block {
    pub hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub prev_hash: Vec<u8>,
    pub nonce: u64,
    difficulty: u128,
}

impl Block {
    /// Returns a new unmined block
    pub fn new(transactions: Vec<Transaction>, prev_hash: Vec<u8>) -> Self {
        Block {
            hash: vec![0; 32],
            transactions,
            prev_hash,
            nonce: 0,
            difficulty: DIFFICULTY,
        }
    }

    /// Returns a new mined block
    ///
    /// Equivalent to calling b = Block::new() followed by b.mine()
    pub fn create(transactions: Vec<Transaction>, prev_hash: Vec<u8>) -> Self {
        let mut block = Block::new(transactions, prev_hash);
        block.mine();
        block
    }

    /// Creates a genesis block
    pub fn create_genesis_block(coinbase: Transaction) -> Block {
        Block::new(vec![coinbase], vec![])
    }

    /// Sets the nonce and hash that satisfies the proof of work condition
    pub fn mine(&mut self) {
        for nonce in 0..(u64::max_value()) {
            self.nonce = nonce;
            self.hash = self.hash();
            let hash = &self.hash.to_vec()[0..16];

            if u128::from_be_bytes(hash.try_into().expect("Cannot convert &[u8] to [u8; 16]"))
                < self.difficulty
            {
                return;
            }
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
    Block Hash: {}
    Prev Hash: {}
    Nonce: {}
    Transactions: {}
",
            hex::encode(&self.hash),
            hex::encode(&self.prev_hash),
            self.nonce,
            self.transactions
                .iter()
                .map(|txn| format!("{}", txn))
                .collect::<String>()
        )
    }
}

impl Hashable for Block {
    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // TODO: Change to functional style (using map and collect into Vec)
        let mut txn_hashes = Vec::<Bytes>::new();
        for txn in self.transactions.iter() {
            txn_hashes.push(txn.encode());
        }

        let tree = blockchain::merkle::MerkleTree::new(txn_hashes);

        bytes.extend(&self.prev_hash);
        bytes.extend(tree.root.data);
        bytes.extend(&self.nonce.to_le_bytes());
        bytes.extend(&self.difficulty.to_le_bytes());

        bytes
    }
}
