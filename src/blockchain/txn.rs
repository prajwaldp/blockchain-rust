use crate::blockchain::wallet::Wallet;
use crate::util::traits::Hashable;
use crate::util::types::Bytes;

use serde::Serialize;

// TODO: Move constants to a module
const CHECKSUM_LENGTH: usize = 4;

#[derive(Clone, Debug, Serialize)]
pub struct TxnInput {
    pub id: Bytes,         // the hash of the transaction
    pub out: i32,          // index where the output appears
    pub signature: Bytes,  // similar to pub_key
    pub public_key: Bytes, // public key that hasn't been hashed
}

impl Hashable for TxnInput {
    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(&self.id);
        bytes.extend(&self.out.to_le_bytes());
        bytes.extend(&self.signature);
        bytes.extend(&self.public_key);
        bytes
    }
}

impl TxnInput {
    pub fn is_uses_key(&self, public_key_hash: &Bytes) -> bool {
        let locking_hash = Wallet::generate_sha256_ripemd160_hash(&self.public_key);
        locking_hash == *public_key_hash
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TxnOutput {
    pub value: i32,
    pub public_key_hash: Bytes, // needed to unlock the tokens in the `value` field
}

impl Hashable for TxnOutput {
    fn encode(&self) -> Bytes {
        let mut bytes: Bytes = Vec::new();

        bytes.extend(&self.value.to_le_bytes());
        bytes.extend(&self.public_key_hash);
        bytes
    }
}

impl TxnOutput {
    pub fn new(value: i32, address: &Bytes) -> Self {
        let mut txn_output = Self {
            value,
            public_key_hash: vec![],
        };

        txn_output.lock(address);
        txn_output
    }

    pub fn lock(&mut self, address: &Bytes) {
        let decoded_address = bs58::decode(address).into_vec().unwrap();
        let public_key_hash = &decoded_address[1..(decoded_address.len() - CHECKSUM_LENGTH)];
        self.public_key_hash = public_key_hash.to_owned();
    }

    pub fn is_locked_with_key(&self, public_key_hash: &Bytes) -> bool {
        self.public_key_hash == *public_key_hash
    }
}
