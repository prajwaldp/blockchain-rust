mod blockchain;
mod network;
mod util;

use riker::actors::*;
use std::time::Duration;

use network::node::*;

fn main() {
    let sys = ActorSystem::new().unwrap();
    let node1 = sys.actor_of_args::<Node, _>("node1", "John").unwrap();

    node1.tell(CreateBlockchain, None);
    node1.tell(
        AddTransactionAndMine {
            from: "John",
            to: "Jane",
            amt: 50,
        },
        None,
    );

    std::thread::sleep(Duration::from_millis(1500));
}
