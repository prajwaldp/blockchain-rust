// mod blockchain;
// mod network;
// mod tests;
// mod util;

// use actix::{Actor, System};
// use network::node::Node;
// use network::Network;

// const N_NODES: u32 = 10;
// const DIFFICULTY: u128 = 0x0000ffffffffffffffffffffffffffff;

// fn main() {
//     let system = System::new("Blockchain Network");
//     println!("Started system {:?}", system);

//     let mut network = Network::new();

//     for _ in 1..=N_NODES {
//         let addr = Node::new(genesis_block.clone()).start();
//         network.nodes.push(addr);
//     }

//     system.run().expect("System did not run");
// }

mod blockchain;
mod util;

fn main() {
    let mut bc = blockchain::BlockChain::new("John");

    let txn1 = blockchain::transaction::Transaction::new("John", "Jane", 50, &bc);
    let blk1 = blockchain::block::Block::create(vec![txn1], bc.last_hash.clone());
    bc.add_block(blk1);

    let txn2 = blockchain::transaction::Transaction::new("John", "Jane", 50, &bc);
    let blk2 = blockchain::block::Block::create(vec![txn2], bc.last_hash.clone());
    bc.add_block(blk2);

    let txn3 = blockchain::transaction::Transaction::new("Jane", "Joe", 10, &bc);
    let blk3 = blockchain::block::Block::create(vec![txn3], bc.last_hash.clone());
    bc.add_block(blk3);

    let txn4 = blockchain::transaction::Transaction::new("Jane", "Joe", 45, &bc);
    let blk4 = blockchain::block::Block::create(vec![txn4], bc.last_hash.clone());
    bc.add_block(blk4);

    bc.print();
}
