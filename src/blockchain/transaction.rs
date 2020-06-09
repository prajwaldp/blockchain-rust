use crate::blockchain::txn::{TxnInput, TxnOutput};
use crate::blockchain::wallet::Wallet;
use crate::blockchain::BlockChain;
use crate::util::traits::Hashable;

use rand::prelude::*;
use secp256k1::{Message, Secp256k1};
use std::collections::HashMap;

const COINBASE_REWARD: i32 = 20;

type Bytes = Vec<u8>;

#[derive(Clone, Debug)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub inputs: Vec<TxnInput>,
    pub outputs: Vec<TxnOutput>,
}

impl Hashable for Transaction {
    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(&self.id);
        bytes.extend(
            self.inputs
                .iter()
                .flat_map(|input| input.encode())
                .collect::<Vec<u8>>(),
        );
        bytes.extend(
            self.outputs
                .iter()
                .flat_map(|output| output.encode())
                .collect::<Vec<u8>>(),
        );

        bytes
    }
}

impl Transaction {
    pub fn new(from: &Bytes, to: &Bytes, amount: i32, chain: &BlockChain) -> Self {
        // Validate the `from` and the `to` addresses
        if !Wallet::is_address_valid(from) {
            eprintln!("Address {} is not a valid address.", hex::encode(from));
        }

        if !Wallet::is_address_valid(to) {
            eprintln!("Address {} is not a valid address.", hex::encode(to));
        }

        let mut inputs: Vec<TxnInput> = Vec::new();
        let mut outputs: Vec<TxnOutput> = Vec::new();

        let wallet_data = Wallet::from_address(from);
        let public_key_hash = wallet_data.public_key_hash;
        let public_key = wallet_data.public_key;
        let private_key = secp256k1::key::SecretKey::from_slice(&wallet_data.private_key).unwrap();

        let (acc, valid_outputs) = chain.find_spendable_outputs(&public_key_hash, amount);

        if acc < amount {
            panic!("Error: Not Enough Funds");
        }

        let mut txn_id;

        for (txn, outs) in &valid_outputs {
            txn_id = hex::decode(txn).expect("Hex decode failed");

            for out in outs {
                let input = TxnInput {
                    id: txn_id.clone(),
                    out: out.clone(),
                    signature: vec![],
                    public_key: public_key.clone(),
                };
                inputs.push(input);
            }
        }

        outputs.push(TxnOutput::new(amount, &to));

        if acc > amount {
            outputs.push(TxnOutput::new(acc - amount, from));
        }

        let mut txn = Transaction {
            id: vec![0; 32],
            inputs,
            outputs,
        };
        txn.id = txn.hash();
        chain.sign_transaction(&mut txn, private_key);

        txn
    }

    /// Create a coinbase transaction, i.e. the first transaction for the
    /// genesis block
    pub fn create_coinbase_txn(to: &Bytes) -> Self {
        let mut rng = rand::thread_rng();

        // Coinbase transaction have random data
        let mut data: Bytes = vec![];
        for _ in 0..24 {
            let i: u8 = rng.gen();
            data.push(i);
        }

        let txin = TxnInput {
            id: vec![],
            out: -1,
            signature: vec![],
            public_key: data,
        };

        let txout = TxnOutput::new(COINBASE_REWARD, &to);

        let mut transaction = Transaction {
            id: vec![],
            inputs: vec![txin],
            outputs: vec![txout],
        };

        transaction.id = transaction.hash();
        transaction
    }

    /// Check whether a transaction is a coinbase transaction.
    ///
    /// Coinbase transactions have one TxnInput, whose `id` is an empty vector, and
    /// `out` is -1
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].id.len() == 0 && self.inputs[0].out == -1
    }

    pub fn sign(
        &mut self,
        private_key: secp256k1::SecretKey,
        prev_txns: HashMap<String, &Transaction>,
    ) -> Result<(), String> {
        if self.is_coinbase() {
            return Ok(());
        }

        for input in &self.inputs {
            if !prev_txns.contains_key(&hex::encode(&input.id)) {
                return Err("The transaction is not present in the history".to_string());
            }
        }

        let mut txn_copy = self.clone();
        let secp = Secp256k1::new();

        for (input_idx, input_data) in self.inputs.iter_mut().enumerate() {
            let prev_txn = &prev_txns[&hex::encode(&input_data.id)];
            txn_copy.inputs[input_idx].signature = vec![];
            txn_copy.inputs[input_idx].public_key = prev_txn.outputs[input_data.out as usize]
                .public_key_hash
                .clone();
            txn_copy.id = txn_copy.hash();
            txn_copy.inputs[input_idx].public_key = vec![];

            let message = Message::from_slice(&txn_copy.id).unwrap();
            let signature = secp.sign(&message, &private_key);

            // It was self.inputs[input_idx]
            input_data.signature = signature.serialize_compact().to_vec();
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn verify(&mut self, prev_txns: HashMap<String, &Transaction>) -> Result<bool, &str> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for input in &self.inputs {
            if !prev_txns.contains_key(&hex::encode(&input.id)) {
                return Err("The transaction is not present in the history");
            }
        }

        let mut txn_copy = self.clone();
        let secp = Secp256k1::new();

        for (input_idx, input_data) in self.inputs.iter().enumerate() {
            let prev_txn = &prev_txns[&hex::encode(&input_data.id)];
            txn_copy.inputs[input_idx].signature = vec![];
            txn_copy.inputs[input_idx].public_key = prev_txn.outputs[input_data.out as usize]
                .public_key_hash
                .clone();
            txn_copy.id = txn_copy.hash();
            txn_copy.inputs[input_idx].public_key = vec![];

            let message = Message::from_slice(&txn_copy.id).unwrap();
            let signature = secp256k1::Signature::from_compact(&input_data.signature).unwrap();
            let public_key = secp256k1::PublicKey::from_slice(&input_data.public_key).unwrap();

            match secp.verify(&message, &signature, &public_key) {
                Ok(_) => (),
                Err(_) => return Err("secp256k1 error"),
            }
        }

        Ok(true)
    }
}

impl std::fmt::Display for TxnInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
                ID: {},
                Out: {},
                Signature: {},
                Public Key: {}
            ",
            hex::encode(&self.id),
            self.out,
            hex::encode(&self.signature),
            hex::encode(&self.public_key)
        )
    }
}

impl std::fmt::Display for TxnOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
                Val: {},
                Public Key Hash: {}
            ",
            self.value,
            hex::encode(&self.public_key_hash)
        )
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
        Transaction ID: {}
            Inputs: {}
            Outputs: {}
            ",
            hex::encode(&self.id),
            self.inputs
                .iter()
                .map(|ip| format!("{}", ip))
                .collect::<String>(),
            self.outputs
                .iter()
                .map(|op| format!("{}", op))
                .collect::<String>()
        )
    }
}
