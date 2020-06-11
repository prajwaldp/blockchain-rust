pub mod node {

    use crate::blockchain::block::Block;
    use crate::blockchain::transaction::Transaction;
    use crate::blockchain::BlockChain;

    use actix::prelude::*;
    use log::{info, trace, warn};

    type Bytes = Vec<u8>;

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
    }

    pub enum GenericResponse {
        OK,
    }

    #[derive(Message)]
    #[rtype(result = "Result<GenericResponse, String>")]
    pub struct GenericMessage(pub Payload);

    pub struct Node {
        pub address: String,
        pub known_nodes: Vec<Recipient<GenericMessage>>,
        pub blockchain: BlockChain,
    }

    impl Node {
        pub fn default(address: String) -> Self {
            Node {
                address,
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
            let block = Block::create(
                vec![txn, coinbase_txn],
                self.blockchain.length,
                self.blockchain.last_hash.clone(),
            );
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
                }

                Payload::UpdateBlockchainFromKnownNodes => {
                    let random_node = &self.known_nodes[0];

                    random_node
                        .try_send(GenericMessage(Payload::RequestBlockchain {
                            sender_addr: ctx.address(),
                        }))
                        .expect("Could not send the get blockchain request")
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
                        info!(
                            "[{}] Received a fresher blockchain. Before Update: Blockchain length = {}",
                            self.address,
                            self.blockchain.length,
                        );
                        self.blockchain = blockchain;
                        info!(
                            "[{}] After Update: Blockchain length = {}",
                            self.address, self.blockchain.length,
                        );
                    }
                }

                Payload::Block { block } => {
                    if block.index == self.blockchain.length
                        && block.timestamp >= self.blockchain.blocks.last().unwrap().timestamp
                        && block.prev_hash == self.blockchain.last_hash
                    {
                        info!(
                            "[{}] Received a valid block to add to the blockchain",
                            &self.address
                        );

                        self.blockchain.add_block(block);
                    } else {
                        warn!(
                            "[{}] Cannot add the block {} (index: {}, timestamp: {}) to blockchain with last hash {} and length {}",
                            &self.address,
                            hex::encode(&block.hash),
                            &block.index, &block.timestamp,
                            hex::encode(&self.blockchain.last_hash),
                            &self.blockchain.length
                        );
                    }
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
}
