use crate::blockchain::block::Block;
use crate::blockchain::transaction::Transaction;
use crate::blockchain::BlockChain;
use crate::broadcast;
use crate::network::server::{Server, ServerMessage};
use crate::util::types::Bytes;

use actix::prelude::*;
use log::{info, trace};
use std::time::Instant;

#[derive(Debug)]
pub enum Events {
    CreatedBlockchain = 0,
    MinedTransaction = 1,
    UpdatedRoutingInfo = 2,
    DownloadedBlockchain = 3,
    ReceivedFresherBlockchain = 4,
    ReceivedNewBlock = 5,
}

// Refactor: semantically order message types in enums
#[derive(Debug)]
pub enum Payload {
    CreateBlockchain {
        address: Bytes,
    },

    UpdateRoutingInfo {
        addresses: Vec<Recipient<GenericMessage>>,
    },

    AddTransactionAndMine {
        from: Bytes,
        to: Bytes,
        amt: i32,
    },

    #[allow(dead_code)]
    PrintInfo,

    #[allow(dead_code)]
    UpdateBlockchainFromKnownNodes,

    RequestBlockchain {
        sender_addr: actix::Addr<Node>,
    },

    Blockchain {
        blockchain: BlockChain,
    },

    Block {
        block: Block,
    },

    PrintWalletBalance {
        public_key_hash: Bytes,
    },
}

pub enum GenericResponse {
    OK,
}

#[derive(Message)]
#[rtype(result = "Result<GenericResponse, String>")]
pub struct GenericMessage(pub Payload);

pub struct Node {
    pub address: String,
    pub server_addr: Addr<Server>,
    pub known_nodes: Vec<Recipient<GenericMessage>>,
    pub blockchain: BlockChain,
}

impl Node {
    pub fn default(address: String, server_addr: Addr<Server>) -> Self {
        Node {
            address,
            server_addr,
            known_nodes: vec![],
            blockchain: BlockChain::new_placeholder(),
        }
    }

    pub fn create_blockchain(&mut self, address: &Bytes) {
        self.blockchain = BlockChain::new(address);
    }

    pub fn make_transaction_and_mine(&mut self, from: Bytes, to: Bytes, amount: i32) {
        let txn = Transaction::new(&from, &to, amount, &self.blockchain);
        let coinbase_txn = Transaction::create_coinbase_txn(&from);

        info!("[{}] Mining new block", &self.address);
        let now = Instant::now();

        let block = Block::create(
            vec![txn, coinbase_txn],
            self.blockchain.length,
            self.blockchain.last_hash.clone(),
        );

        info!(
            "[{}] Mined successfully in {} seconds",
            &self.address,
            now.elapsed().as_secs()
        );

        let payload = format!(
            r#"{{
                "nodeId":"{}",
                "eventId":"{:?}",
                "details": {{
                    "timeTaken": {}
                }}
            }}"#,
            &self.address,
            Events::MinedTransaction,
            now.elapsed().as_secs()
        );

        broadcast!(self.server_addr, payload);

        self.blockchain.add_block(block.clone());

        for addr in self.known_nodes.iter() {
            trace!(
                "[{}] Sending block to {:?} in `make_transaction_and_mine`",
                &self.address,
                &addr
            );
            addr.try_send(GenericMessage(Payload::Block {
                block: block.clone(),
            }))
            .expect(&format!("Couldn't send updated block to {:?}", addr));
        }
    }
}

impl Actor for Node {
    type Context = Context<Self>;
}

impl Handler<GenericMessage> for Node {
    type Result = Result<GenericResponse, String>;

    fn handle(&mut self, msg: GenericMessage, ctx: &mut Context<Self>) -> Self::Result {
        trace!("[{}] Received {:?}", self.address, msg.0);

        match msg.0 {
            Payload::CreateBlockchain { address } => {
                self.create_blockchain(&address);

                for addr in self.known_nodes.iter() {
                    addr.try_send(GenericMessage(Payload::Blockchain {
                        blockchain: self.blockchain.clone(),
                    }))
                    .expect(&format!("Couldn't send blockchain to {:?}", addr));
                }

                let j = serde_json::to_string(&self.blockchain).unwrap();

                let payload = format!(
                    r#"{{
                        "nodeId":"{}",
                        "eventId":"{:?}",
                        "details":{{
                            "rawBlockchainData": {}
                        }}
                    }}"#,
                    &self.address,
                    Events::CreatedBlockchain,
                    j
                );

                broadcast!(self.server_addr, payload);
            }

            Payload::UpdateRoutingInfo { addresses } => {
                // Remove the nodes own addresss if present
                let filtered_addresses = addresses
                    .into_iter()
                    .filter(|a| a != &ctx.address().recipient())
                    .collect::<Vec<_>>();

                self.known_nodes = filtered_addresses;
                info!(
                    "[{}] Update address list with {} nodes",
                    self.address,
                    self.known_nodes.len()
                );

                let payload = format!(
                    r#"{{
                        "nodeId":"{}",
                        "eventId":"{:?}",
                        "details":{{
                            "neighbourCount": {}
                        }}
                    }}"#,
                    &self.address,
                    Events::UpdatedRoutingInfo,
                    format!("{:?}", self.known_nodes.len())
                );

                broadcast!(self.server_addr, payload);
            }

            Payload::UpdateBlockchainFromKnownNodes => {
                let random_node = &self.known_nodes[0];

                random_node
                    .try_send(GenericMessage(Payload::RequestBlockchain {
                        sender_addr: ctx.address(),
                    }))
                    .expect("Could not send the get blockchain request");

                let j = serde_json::to_string(&self.blockchain).unwrap();

                let payload = format!(
                    r#"{{
                        "nodeId":"{}",
                        "eventId":"{:?}",
                        "details":{{
                            "rawBlockchainData": {}
                        }}
                    }}"#,
                    &self.address,
                    Events::DownloadedBlockchain,
                    j
                );

                broadcast!(self.server_addr, payload);
            }

            Payload::PrintInfo => {
                println!("{:?}", self);
                println!("{}", self.blockchain);
            }

            Payload::AddTransactionAndMine { from, to, amt } => {
                self.make_transaction_and_mine(from.clone(), to.clone(), amt);
            }

            Payload::RequestBlockchain { sender_addr } => {
                sender_addr
                    .try_send(GenericMessage(Payload::Blockchain {
                        blockchain: self.blockchain.clone(),
                    }))
                    .expect(&format!("Couldn't send blockchain to {:?}", sender_addr));
            }

            Payload::Blockchain { blockchain } => {
                if self.blockchain.blocks.len() < blockchain.blocks.len() {
                    let old_blockchain_length = self.blockchain.length;

                    info!(
                        "[{}] Received a fresher blockchain. Before Update: Blockchain length = {}",
                        self.address, old_blockchain_length,
                    );

                    self.blockchain = blockchain;
                    let new_blockchain_length = self.blockchain.length;

                    info!(
                        "[{}] After Update: Blockchain length = {}",
                        self.address, self.blockchain.length,
                    );

                    let j = serde_json::to_string(&self.blockchain).unwrap();

                    let payload = format!(
                        r#"{{
                            "nodeId":"{}",
                            "eventId":"{:?}",
                            "details":{{
                                "oldLength": {},
                                "newLength": {},
                                "rawBlockchainData": {}
                            }}
                        }}"#,
                        &self.address,
                        Events::ReceivedFresherBlockchain,
                        old_blockchain_length,
                        new_blockchain_length,
                        j
                    );

                    broadcast!(self.server_addr, payload);
                }
            }

            Payload::Block { block } => {
                info!(
                    "[{}] Received a block to add to the blockchain",
                    &self.address
                );
                self.blockchain.add_block_to_memory_pool(block);

                let payload = format!(
                    r#"{{
                        "nodeId":"{}",
                        "eventId":"{:?}"
                    }}"#,
                    &self.address,
                    Events::ReceivedNewBlock
                );

                broadcast!(self.server_addr, payload);
            }

            Payload::PrintWalletBalance { public_key_hash } => {
                let unspent_txns = self.blockchain.find_unspent_transactions(&public_key_hash);
                let mut balance: i32 = 0;
                for txn in unspent_txns.iter() {
                    for output in txn.outputs.iter() {
                        if output.is_locked_with_key(&public_key_hash) {
                            balance += output.value;
                        }
                    }
                }
                println!("{}", balance);
            }
        }
        Ok(GenericResponse::OK)
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node: {}\nKnown Addresses: {}",
            self.address,
            self.known_nodes
                .iter()
                .map(|addr| format!("{:?}", addr))
                .collect::<String>()
        )
    }
}
