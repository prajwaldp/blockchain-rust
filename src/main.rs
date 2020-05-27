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
mod network;
mod util;

use network::node::Node;
use network::Network;

fn main() {
    let mut network = Network::new();

    let mut n1 = Node::new("John");
    let mut n2 = Node::new("Jane");
    let mut n3 = Node::new("Joe");

    n1.join_network(&mut network);
    n1.create_blockchain();

    n1.make_transaction_and_mine("Jane", 50);

    println!("{}", n1.blockchain);

    n2.join_network(&mut network);
    n3.join_network(&mut network);

    // let mut bc = blockchain::BlockChain::new("John");
    // bc.add_transaction("John", "Jane", 50);
    // bc.add_transaction("John", "Jane", 50);
    // bc.add_transaction("Jane", "Joe", 10);
    // bc.add_transaction("Jane", "Joe", 45);
    // println!("{}", bc);
}
