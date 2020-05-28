pub mod node {

    use crate::blockchain::block::Block;
    use crate::blockchain::transaction::Transaction;
    use crate::blockchain::BlockChain;

    use riker::actors::*;

    #[derive(Clone, Debug)]
    pub struct CreateBlockchain;

    #[derive(Clone, Debug)]
    pub struct AddTransactionAndMine {
        pub from: &'static str,
        pub to: &'static str,
        pub amt: i32,
    }

    #[actor(CreateBlockchain, AddTransactionAndMine)]
    pub struct Node {
        pub address: &'static str,
        pub known_nodes: Vec<Node>,
        pub blockchain: BlockChain,
    }

    impl Node {
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
        type Msg = NodeMsg;

        fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
            self.receive(ctx, msg, sender);
        }
    }

    impl ActorFactoryArgs<&'static str> for Node {
        fn create_args(address: &'static str) -> Self {
            Node {
                address,
                known_nodes: vec![],
                blockchain: BlockChain::new_placeholder(),
            }
        }
    }

    impl Receive<CreateBlockchain> for Node {
        type Msg = NodeMsg;

        fn receive(&mut self, _ctx: &Context<Self::Msg>, _msg: CreateBlockchain, _sender: Sender) {
            self.create_blockchain();
            println!("Created blockchain");
            println!("{}", self.blockchain);
        }
    }

    impl Receive<AddTransactionAndMine> for Node {
        type Msg = NodeMsg;

        fn receive(
            &mut self,
            _ctx: &Context<Self::Msg>,
            msg: AddTransactionAndMine,
            _sender: Sender,
        ) {
            self.make_transaction_and_mine(msg.from, msg.to, msg.amt);
            println!("Mined transaction");
            println!("{}", self.blockchain);
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
