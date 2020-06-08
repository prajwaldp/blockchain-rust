pub mod node {

    use crate::blockchain::block::Block;
    use crate::blockchain::transaction::Transaction;
    use crate::blockchain::BlockChain;

    use actix::prelude::*;

    type Bytes = Vec<u8>;

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
        PrintInfo,
        UpdateBlockchainFromKnownNodes,
        RequestBlockchain {
            sender_addr: actix::Addr<Node>,
        },
        Blockchain {
            blockchain: BlockChain,
        },
    }

    pub enum GenericResponse {
        OK,
        BlockChain(BlockChain),
    }

    #[derive(Message)]
    #[rtype(result = "Result<GenericResponse, String>")]
    pub struct GenericMessage(pub Payload);

    // #[derive(Message)]
    // #[rtype(result = "Result<BlockChain, String>")]
    // pub struct DownloadBlockChainRequest;

    pub struct Node {
        pub address: &'static str,
        pub known_nodes: Vec<Recipient<GenericMessage>>,
        pub blockchain: BlockChain,
    }

    impl Node {
        pub fn default(address: &'static str) -> Self {
            Node {
                address,
                known_nodes: vec![],
                blockchain: BlockChain::new_placeholder(),
            }
        }

        // pub async fn new(
        //     address: &'static str,
        //     known_nodes: Vec<Recipient<GenericMessage>>,
        // ) -> Self {
        //     let mut node = Node {
        //         address,
        //         known_nodes,
        //         blockchain: BlockChain::new_placeholder(),
        //     };

        //     for known_node in &node.known_nodes {
        //         let actix_result = known_node.send(DownloadBlockChainRequest).await;

        //         match actix_result {
        //             Ok(result) => match result {
        //                 Ok(blockchain) => {
        //                     node.blockchain = blockchain;
        //                     println!("Downloaded blockchain successfully");
        //                     break;
        //                 }
        //                 Err(err) => println!(
        //                     "[Error] DownloadBlockChainRequest responsed to with {}",
        //                     err
        //                 ),
        //             },

        //             Err(err) => {
        //                 println!("[Error] Actix could not send message, failed with {}", err)
        //             }
        //         }
        //     }

        //     node
        // }

        pub fn create_blockchain(&mut self, address: &Bytes) {
            self.blockchain = BlockChain::new(address);
        }

        pub fn make_transaction_and_mine(&mut self, from: Bytes, to: Bytes, amount: i32) {
            // let from = from.as_bytes().to_owned();
            // let to = to.as_bytes().to_owned();

            let txn = Transaction::new(&from, &to, amount, &self.blockchain);
            let coinbase_txn = Transaction::create_coinbase_txn(&from);
            println!("called 1");
            let block = Block::create(vec![txn, coinbase_txn], self.blockchain.last_hash.clone());
            println!("called 2");
            self.blockchain.add_block(block);
        }
    }

    impl Actor for Node {
        type Context = Context<Self>;
    }

    impl Handler<GenericMessage> for Node {
        type Result = Result<GenericResponse, String>;

        fn handle(&mut self, msg: GenericMessage, ctx: &mut Context<Self>) -> Self::Result {
            println!("[{}]: Received {:?}", self.address, msg.0);
            match msg.0 {
                Payload::CreateBlockchain { address } => {
                    self.create_blockchain(&address);
                    println!("{}", self.blockchain);
                }

                Payload::AddTransactionAndMine { from, to, amt } => {
                    self.make_transaction_and_mine(from.clone(), to.clone(), amt);
                    println!("Mined transaction");
                    println!("{}", self.blockchain);
                }

                Payload::UpdateRoutingInfo { addresses } => {
                    self.known_nodes = addresses;
                }

                Payload::PrintInfo => {
                    println!("{:?}", self);

                    for addr in self.known_nodes.iter() {
                        println!("{:?}", addr);
                    }
                }

                Payload::UpdateBlockchainFromKnownNodes => {
                    let random_node = &self.known_nodes[0];

                    random_node
                        .try_send(GenericMessage(Payload::RequestBlockchain {
                            sender_addr: ctx.address(),
                        }))
                        .expect("Could not send the get blockchain request")
                }

                Payload::RequestBlockchain { sender_addr } => {
                    sender_addr
                        .try_send(GenericMessage(Payload::Blockchain {
                            blockchain: self.blockchain.clone(),
                        }))
                        .expect(&format!("Couldn't send blockchain to {:?}", sender_addr));
                }

                Payload::Blockchain { blockchain } => {
                    self.blockchain = blockchain;
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
