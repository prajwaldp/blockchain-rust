use crate::blockchain::transaction::Transaction;
use crate::blockchain::{block::Block, BlockChain};
use actix::{Actor, Context, Handler, System};

#[derive(Debug)]
pub struct Node {
    pub neighbors: Vec<Node>,
    pub blockchain: BlockChain,
}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("I am alive!");
        for block in &self.blockchain.blocks {
            println!("{:?}", block);
        }
        System::current().stop(); // <- stop system
    }
}

impl Handler<Block> for Node {
    type Result = usize; // <- Message response type

    fn handle(&mut self, msg: Block, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?} received Block {:?}", self, msg);
        0
    }
}

impl Handler<Transaction> for Node {
    type Result = usize; // <- Message response type

    fn handle(&mut self, msg: Transaction, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?} received Transaction{:?}", self, msg);
        0
    }
}

impl Node {
    pub fn new(genesis_block: Block) -> Self {
        let mut blockchain = BlockChain::new();
        blockchain
            .update_with_block(genesis_block)
            .expect("Could not create blockchain");

        Node {
            neighbors: vec![],
            blockchain: blockchain,
        }
    }
}
