mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use blockchain::wallet::Wallet;
use network::node::*;

fn handle_result<T, E: std::fmt::Display>(result: Result<T, E>, desc: &'static str) {
    match result {
        Ok(_) => (),
        Err(err) => println!("[Error] {} responsed to with {}", desc, err),
    }
}

#[actix_rt::main]
async fn main() {
    let addr1 = Node::default("Node1").start();
    let wallet1 = Wallet::new();

    let result = addr1
        .send(GenericMessage(Payload::CreateBlockchain {
            address: wallet1.address.clone(),
        }))
        .await;

    handle_result(result, "CreateBlockchain");

    let wallet2 = Wallet::new();
    let result = addr1
        .send(GenericMessage(Payload::AddTransactionAndMine {
            from: wallet1.address.clone(),
            to: wallet2.address.clone(),
            amt: 10,
        }))
        .await;

    handle_result(result, "AddTransactionAndMine");

    let addr2 = Node::default("Node2").start();
    let result = addr2
        .send(GenericMessage(Payload::UpdateRoutingInfo {
            addresses: vec![addr1.recipient().clone()],
        }))
        .await;

    handle_result(result, "UpdateRoutingInfo");

    let result = addr2.send(GenericMessage(Payload::PrintInfo)).await;
    handle_result(result, "PrintInfo");

    let result = addr2
        .send(GenericMessage(Payload::UpdateBlockchainFromKnownNodes))
        .await;
    handle_result(result, "PrintInfo");

    System::current().stop();
}
