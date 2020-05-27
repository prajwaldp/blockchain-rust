pub mod node;

use node::Node;

pub struct Network {
    nodes: Vec<Node>,
}

impl Network {
    pub fn new() -> Self {
        Network { nodes: vec![] }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }
}
