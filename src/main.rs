mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use log::*;
use simplelog::*;
use std::env;
use std::fs::File;

use blockchain::wallet::Wallet;
use network::node::*;
use util::helper_functions::handle_result;

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

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("\nUsage: {} number-of-nodes number-of-wallets", args[0]);
        std::process::exit(0);
    }

    let n_nodes: u32 = args[1].parse::<u32>().expect("Couldn't parse n_nodes");
    let n_wallets: u32 = args[2].parse::<u32>().expect("Couldn't parse n_wallets");

    println!("Running the simulation with:");
    println!("Nodes: {}", n_nodes);
    println!("Wallets: {}\n", n_wallets);

    let mut nodes: Vec<Addr<Node>> = Vec::new();
    let mut wallets: Vec<Wallet> = Vec::new();

    for i in 0..n_nodes {
        let node_name = format!("Node-{}", i);
        let addr = Node::default(node_name).start();
        nodes.push(addr);
    }

    let recepient_addresses = nodes
        .iter()
        .map(|n| n.clone().recipient())
        .collect::<Vec<Recipient<GenericMessage>>>();

    // Creating a full network
    for node in nodes.iter() {
        let res = node
            .send(GenericMessage(Payload::UpdateRoutingInfo {
                addresses: recepient_addresses.clone(),
            }))
            .await;
        handle_result(res, "UpdateRoutingInfo");
    }

    for _ in 0..n_wallets {
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

    // let result = nodes[1]
    //     .send(GenericMessage(Payload::UpdateBlockchainFromKnownNodes))
    //     .await;
    // handle_result(result, "UpdateBlockchainFromKnownNodes");

    // let result = nodes[1].send(GenericMessage(Payload::PrintInfo)).await;
    // handle_result(result, "PrintInfo");

    System::current().stop();
}
