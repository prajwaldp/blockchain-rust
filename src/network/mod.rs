pub mod node;

use node::Node;

pub struct Network {
    pub nodes: Vec<actix::Addr<Node>>,
}

impl Network {
    pub fn new() -> Self {
        Network { nodes: vec![] }
    }
}
