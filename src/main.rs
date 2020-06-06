mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use blockchain::wallet::Wallet;
use network::node::*;

#[actix_rt::main]
async fn main() {
    let addr = Node::default("John").start();

    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();

    let result = addr
        .send(CreateBlockchain {
            address: wallet1.address.clone(),
        })
        .await;

    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] CreateBlockChain responsed to with {}", err),
    }

    let result = addr
        .send(AddTransactionAndMine {
            from: wallet1.address.clone(),
            to: wallet2.address.clone(),
            amt: 10,
        })
        .await;

    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] CreateBlockChain responded to with {}", err),
    }

    // Let's try downloading the blockchain from another node
    let new_node = Node::new("Jane", vec![addr.recipient()]).await;
    new_node.start();

    System::current().stop();
}
