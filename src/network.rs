pub mod node {

    use crate::blockchain::block::Block;
    use crate::blockchain::transaction::Transaction;
    use crate::blockchain::BlockChain;

    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "Result<(), String>")]
    pub struct CreateBlockchain;

    #[derive(Message)]
    #[rtype(result = "Result<(), String>")]
    pub struct AddTransactionAndMine {
        pub from: &'static str,
        pub to: &'static str,
        pub amt: i32,
    }

    #[derive(Message)]
    #[rtype(result = "Result<BlockChain, String>")]
    pub struct DownloadBlockChainRequest;

    pub struct Node {
        pub address: &'static str,
        pub known_nodes: Vec<Recipient<DownloadBlockChainRequest>>,
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

        pub async fn new(
            address: &'static str,
            known_nodes: Vec<Recipient<DownloadBlockChainRequest>>,
        ) -> Self {
            let mut node = Node {
                address,
                known_nodes,
                blockchain: BlockChain::new_placeholder(),
            };

            for known_node in &node.known_nodes {
                let actix_result = known_node.send(DownloadBlockChainRequest).await;

                match actix_result {
                    Ok(result) => match result {
                        Ok(blockchain) => {
                            node.blockchain = blockchain;
                            println!("Downloaded blockchain successfully");
                            break;
                        }
                        Err(err) => println!(
                            "[Error] DownloadBlockChainRequest responsed to with {}",
                            err
                        ),
                    },

                    Err(err) => {
                        println!("[Error] Actix could not send message, failed with {}", err)
                    }
                }
            }

            node
        }

        pub fn create_blockchain(&mut self) {
            self.blockchain = BlockChain::new(&self.address);
        }

        pub fn make_transaction_and_mine(
            &mut self,
            from: &'static str,
            to: &'static str,
            amount: i32,
        ) {
            let txn = Transaction::new(from, to, amount, &self.blockchain);
            let block = Block::create(vec![txn], self.blockchain.last_hash.clone());
            self.blockchain.add_block(block);
        }
    }

    impl Actor for Node {
        type Context = Context<Self>;
    }

    impl Handler<CreateBlockchain> for Node {
        type Result = Result<(), String>;

        fn handle(&mut self, _msg: CreateBlockchain, _ctx: &mut Context<Self>) -> Self::Result {
            self.create_blockchain();
            println!("{}", self.blockchain);
            Ok(())
        }
    }

    impl Handler<AddTransactionAndMine> for Node {
        type Result = Result<(), String>;

        fn handle(&mut self, msg: AddTransactionAndMine, _ctx: &mut Context<Self>) -> Self::Result {
            self.make_transaction_and_mine(msg.from, msg.to, msg.amt);
            println!("Mined transaction");
            println!("{}", self.blockchain);
            Ok(())
        }
    }

    impl Handler<DownloadBlockChainRequest> for Node {
        type Result = Result<BlockChain, String>;

        fn handle(
            &mut self,
            _msg: DownloadBlockChainRequest,
            _ctx: &mut Context<Self>,
        ) -> Self::Result {
            Ok(self.blockchain.clone())
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
