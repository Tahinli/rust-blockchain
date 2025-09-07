use chrono::Utc;

use crate::block::Block;

#[derive(Debug, Clone)]
pub struct BlockChain {
    pub genesis_block: Block,
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl BlockChain {
    pub fn new(difficulty: usize) -> Self {
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp_millis() as u64,
            proof_of_work: 0_u64,
            previous_hash: String::new(),
            hash: String::new(),
        };

        let chain = vec![genesis_block.clone()];

        let blockchain = BlockChain {
            genesis_block,
            chain,
            difficulty,
        };
        blockchain
    }

    pub fn add_block(&mut self) {
        let new_block = Block::new(
            self.chain.len() as u64,
            self.chain[&self.chain.len() - 1].previous_hash.clone(),
        )
        .mine(self.clone());
        self.chain.push(new_block);
    }
}
