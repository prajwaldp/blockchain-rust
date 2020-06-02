use crate::util::traits::Hashable;

#[derive(Clone, Debug)]
pub struct TxnInput {
    pub id: Vec<u8>,       // the hash of the transaction
    pub out: i32,          // index where the output appears
    pub sig: &'static str, // similar to pub_key
}

impl Hashable for TxnInput {
    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(&self.id);
        bytes.extend(&self.out.to_le_bytes());
        bytes.extend(self.sig.as_bytes());

        bytes
    }
}

impl TxnInput {
    pub fn can_unlock(&self, data: &'static str) -> bool {
        self.sig == data
    }
}

#[derive(Clone, Debug)]
pub struct TxnOutput {
    pub value: i32,
    pub pub_key: &'static str, // needed to unlock the tokens in the `value` field
}

impl Hashable for TxnOutput {
    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(&self.value.to_le_bytes());
        bytes.extend(self.pub_key.as_bytes());

        bytes
    }
}

impl TxnOutput {
    pub fn can_be_unlocked(&self, data: &'static str) -> bool {
        self.pub_key == data
    }
}
