use crate::blockchain::BlockChain;
use crate::util::traits::Hashable;

use actix::Message;

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

#[derive(Clone, Debug, Message)]
#[rtype(result = "usize")]
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
    pub fn new(from: &'static str, to: &'static str, amount: i32, chain: &BlockChain) -> Self {
        let mut inputs: Vec<TxnInput> = Vec::new();
        let mut outputs: Vec<TxnOutput> = Vec::new();

        let (acc, valid_outputs) = chain.find_spendable_outputs(from, amount);

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
                    sig: from.clone(),
                };
                inputs.push(input);
            }
        }

        outputs.push(TxnOutput {
            value: amount,
            pub_key: to.clone(),
        });

        if acc > amount {
            outputs.push(TxnOutput {
                value: acc - amount,
                pub_key: from.clone(),
            });
        }

        let mut txn = Transaction {
            id: vec![0; 32],
            inputs,
            outputs,
        };
        txn.setid();
        txn
    }

    /// Create a coinbase transaction, i.e. the first transaction for the
    /// genesis block
    pub fn create_coinbase_txn(to: &'static str, data: &'static str) -> Self {
        let txin = TxnInput {
            id: vec![],
            out: -1,
            sig: data,
        };

        let txout = TxnOutput {
            value: 100,
            pub_key: to,
        };

        let mut transaction = Transaction {
            id: vec![],
            inputs: vec![txin],
            outputs: vec![txout],
        };

        transaction.setid();
        transaction
    }

    /// Sets the ID of the transaction
    fn setid(&mut self) {
        self.id = self.hash();
    }

    /// Check whether a transaction is a coinbase transaction.
    ///
    /// Coinbase transactions have one TxnInput, whose `id` is an empty vector, and
    /// `out` is -1
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].id.len() == 0 && self.inputs[0].out == -1
    }
}

impl std::fmt::Display for TxnInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
                ID: {},
                Out: {},
                Sig: {},
            ",
            hex::encode(&self.id),
            self.out,
            self.sig
        )
    }
}

impl std::fmt::Display for TxnOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "
                Val: {},
                Pub Key: {}
            ",
            self.value, self.pub_key
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
