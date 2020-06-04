mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use blockchain::wallet::Wallet;
use network::node::*;

#[actix_rt::main]
async fn main() {
    let addr = Node::default("John").start();
    let miner = Wallet::new();
    let person2 = Wallet::new();

    let result = addr
        .send(CreateBlockchain {
            address: miner.address.clone(),
        })
        .await;

    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] CreateBlockChain responsed to with {}", err),
    }

    let result = addr
        .send(AddTransactionAndMine {
            from: miner.address.clone(),
            to: person2.address.clone(),
            amt: 50,
        })
        .await;

    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] CreateBlockChain responsed to with {}", err),
    }

    // Let's try downloading the blockchain from another node

    let node1 = Node::new("Jane", vec![addr.recipient()]).await;
    node1.start();

    System::current().stop();
}
