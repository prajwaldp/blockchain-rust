mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use log::*;
use simplelog::*;
use std::fs::File;

use blockchain::wallet::Wallet;
use network::node::*;
use util::helper_functions::handle_result;

const N_NODES: u32 = 10;
const N_WALLETS: u32 = 10;

#[actix_rt::main]
async fn main() {
    // Initialize the logger
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("log/application.log").unwrap(),
        ),
    ])
    .unwrap();

    let mut nodes: Vec<Addr<Node>> = Vec::new();
    let mut wallets: Vec<Wallet> = Vec::new();

    for i in 0..N_NODES {
        let node_name = format!("Node-{}", i);
        let addr = Node::default(node_name).start();
        nodes.push(addr);
    }

    let recepient_addresses = nodes
        .iter()
        .map(|n| n.clone().recipient())
        .collect::<Vec<Recipient<GenericMessage>>>();

    for node in nodes.iter() {
        let res = node
            .send(GenericMessage(Payload::UpdateRoutingInfo {
                addresses: recepient_addresses.clone(),
            }))
            .await;
        handle_result(res, "UpdateRoutingInfo");
    }

    for _ in 0..N_WALLETS {
        let wallet = Wallet::new();
        wallets.push(wallet);
    }

    let result = nodes[0]
        .send(GenericMessage(Payload::CreateBlockchain {
            address: wallets[0].address.clone(),
        }))
        .await;
    handle_result(result, "CreateBlockchain");

    let result = nodes[0]
        .send(GenericMessage(Payload::AddTransactionAndMine {
            from: wallets[0].address.clone(),
            to: wallets[1].address.clone(),
            amt: 10,
        }))
        .await;
    handle_result(result, "AddTransactionAndMine");

    let result = nodes[0]
        .send(GenericMessage(Payload::AddTransactionAndMine {
            from: wallets[1].address.clone(),
            to: wallets[2].address.clone(),
            amt: 10,
        }))
        .await;
    handle_result(result, "AddTransactionAndMine");

    let result = nodes[1]
        .send(GenericMessage(Payload::UpdateBlockchainFromKnownNodes))
        .await;
    handle_result(result, "UpdateBlockchainFromKnownNodes");

    let result = nodes[1].send(GenericMessage(Payload::PrintInfo)).await;
    handle_result(result, "PrintInfo");

    System::current().stop();
}
