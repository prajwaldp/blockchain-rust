use crate::blockchain::block::Block;
use crate::blockchain::transaction::Transaction;
use crate::blockchain::BlockChain;
use crate::network::Network;

#[derive(Clone)]
pub struct Node {
    pub address: &'static str,
    known_nodes: Vec<Node>,
    pub blockchain: BlockChain,
}

impl Node {
    pub fn new(address: &'static str) -> Self {
        Node {
            address,
            known_nodes: vec![],
            blockchain: BlockChain::new_placeholder(),
        }
    }

    pub fn join_network(&mut self, network: &mut Network) {
        network.add_node(self.clone());
        for node in &network.nodes {
            self.known_nodes.push(node.clone());
        }
    }

    pub fn create_blockchain(&mut self) {
        self.blockchain = BlockChain::new(&self.address);
    }

    pub fn make_transaction_and_mine(&mut self, to: &'static str, amount: i32) {
        let txn = Transaction::new(self.address, to, amount, &self.blockchain);
        let block = Block::create(vec![txn], self.blockchain.last_hash.clone());
        self.blockchain.add_block(block);
    }
}
