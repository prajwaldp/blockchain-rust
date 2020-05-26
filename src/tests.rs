#[cfg(test)]
use crate::*;

#[test]
pub fn test() {
    let difficulty: u128 = 0x0000ffffffffffffffffffffffffffff;
    let mut last_hash;

    let mut genesis_block =
        blockchain::block::Block::new(0, util::time::now(), vec![0; 32], vec![], difficulty);

    genesis_block.mine();
    last_hash = genesis_block.hash.clone();

    println!("{:?}", genesis_block);

    let mut blockchain = blockchain::BlockChain::new();
    blockchain
        .update_with_block(genesis_block)
        .expect("Failed to add genesis block");

    for i in 1..=5 {
        let mut block = blockchain::block::Block::new(
            i,
            util::time::now(),
            last_hash,
            vec![blockchain::transaction::Transaction {
                inputs: vec![],
                outputs: vec![blockchain::transaction::Output {
                    to_addr: "Alice".to_owned(),
                    value: 50,
                }],
            }],
            difficulty,
        );

        block.mine();
        last_hash = block.hash.clone();

        println!("{:?}", block);
        blockchain
            .update_with_block(block)
            .expect("Adding block did not work");
    }
}
