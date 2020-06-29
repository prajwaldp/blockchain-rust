mod blockchain;
mod network;
mod util;

use actix::prelude::*;
use log::*;
use simplelog::*;
use std::env;
use std::fs::File;
use std::{thread, time};

use blockchain::wallet::Wallet;
use network::node::*;
use network::server::{Server, ServerCommand, ServerMessage};
use util::helper_functions::handle_result;

// Alternate macro to send the broadcast request
// let res = $server.try_send(ServerMessage($payload));
// match res {
//     Ok(_) => (),
//     Err(e) => println!("Error: Couldn't send broadcast request: {:?}", e),
// };

#[macro_export]
macro_rules! broadcast {
    ( $server:expr, $payload:expr ) => {
        $server.do_send(ServerMessage($payload));
    };
}

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

    let server_addr = Server::init().start();
    let result = server_addr.try_send(ServerCommand("Listen"));
    handle_result(result, "Error getting the server to listen");

    // Wait for the server to start
    let ten_millis = time::Duration::from_millis(100);
    thread::sleep(ten_millis);

    let mut nodes: Vec<Addr<Node>> = Vec::new();
    let mut wallets: Vec<Wallet> = Vec::new();

    for i in 0..n_nodes {
        let node_name = format!("Node-{}", i);
        let addr = Node::default(node_name.clone(), server_addr.clone()).start();
        nodes.push(addr);

        let payload = format!(
            r#"{{
                "nodeId":"Main",
                "eventId":"SpawnedNode",
                "details":{{
                    "nodeId": "{}"
                }}
            }}"#,
            node_name
        );

        broadcast!(server_addr, payload);
    }

    let recipient_addresses = nodes
        .iter()
        .map(|n| n.clone().recipient())
        .collect::<Vec<Recipient<GenericMessage>>>();

    // Creating a full network
    for node in nodes.iter() {
        let res = node.try_send(GenericMessage(Payload::UpdateRoutingInfo {
            addresses: recipient_addresses.clone(),
        }));
        handle_result(res, "UpdateRoutingInfo");
    }

    for _ in 0..n_wallets {
        let wallet = Wallet::new();
        let wallet_address = wallet.address.clone();
        wallets.push(wallet);

        let payload = format!(
            r#"{{
                "nodeId":"Main",
                "eventId":"CreatedWallet",
                "details":{{
                    "wallletAddress": {:?}
                }}
            }}"#,
            hex::encode(&wallet_address)
        );

        // Using send here instead of the macro broadcast!
        // which uses try_send
        // let _res = server_addr.send(ServerMessage(payload));
        broadcast!(server_addr, payload);
    }

    let result = nodes[0].try_send(GenericMessage(Payload::CreateBlockchain {
        address: wallets[0].address.clone(),
    }));
    handle_result(result, "CreateBlockchain");

    // Simulation Seed Money
    for i in 0..(n_wallets - 1) {
        let result = nodes[0].try_send(GenericMessage(Payload::AddTransactionAndMine {
            from: wallets[i as usize].address.clone(),
            to: wallets[(i + 1) as usize].address.clone(),
            amt: 10,
        }));
        handle_result(result, "AddTransactionAndMine");
    }

    // Get Wallet Balances
    println!("\nWallet Balances");
    println!("===============");
    for public_key_hash in wallets.iter().map(|w| w.public_key_hash.clone()) {
        print!("{} : ", hex::encode(&public_key_hash));
        let result = nodes[1]
            .send(GenericMessage(Payload::PrintWalletBalance {
                public_key_hash: public_key_hash,
            }))
            .await;
        handle_result(result, "PrintWalletBalance")
    }

    System::current().stop();
}
