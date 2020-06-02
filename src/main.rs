mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use network::node::*;

#[actix_rt::main]
async fn main() {
    let addr = Node::default("John").start();
    let result = addr.send(CreateBlockchain).await;

    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] CreateBlockChain responsed to with {}", err),
    }

    let result = addr
        .send(AddTransactionAndMine {
            from: "John",
            to: "Jane",
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

    let wallet = blockchain::wallet::Wallet::new();
    println!("{}", wallet);
}
