use crate::util::types::Bytes;

#[derive(Clone)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub data: Bytes,
}

pub struct MerkleTree {
    pub root: MerkleNode,
}

impl MerkleNode {
    pub fn new(
        left: Option<Box<MerkleNode>>,
        right: Option<Box<MerkleNode>>,
        data: &Bytes,
    ) -> Self {
        let hash: Bytes;

        if left.is_none() && right.is_none() {
            hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &data);
        } else {
            let mut prev_hashes: Bytes = vec![];

            let left_data = left.as_ref().unwrap().data.clone();
            let right_data = right.as_ref().unwrap().data.clone();

            prev_hashes.extend(left_data);
            prev_hashes.extend(right_data);

            hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &prev_hashes);
        }

        MerkleNode {
            left,
            right,
            data: hash,
        }
    }
}

impl MerkleTree {
    pub fn new(mut data: Vec<Bytes>) -> Self {
        let mut nodes = Vec::<MerkleNode>::new();

        if data.len() % 2 != 0 {
            let last_transaction = data[data.len() - 1].clone();
            data.push(last_transaction);
        }

        for row in data.iter() {
            let node = MerkleNode::new(None, None, row);
            nodes.push(node);
        }

        for _ in 0..(data.len() / 2) {
            let mut level = Vec::<MerkleNode>::new();

            for j in (0..data.len()).step_by(2) {
                let left = &nodes[j];
                let right = &nodes[j + 1];

                let node = MerkleNode::new(
                    Some(Box::new(left.clone())),
                    Some(Box::new(right.clone())),
                    &vec![],
                );

                level.push(node);
            }

            nodes = level;
        }

        MerkleTree {
            root: nodes[0].clone(),
        }
    }
}
