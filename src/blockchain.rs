use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

use crate::block::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            data: String::from("Tahinli"),
            proof_of_work: 0_u64,
            previous_hash: String::new(),
            hash: String::new(),
            hash_time_cost: Duration::from_secs(0),
        };

        let chain = vec![genesis_block.clone()];

        BlockChain {
            genesis_block,
            chain,
            difficulty,
        }
    }

    pub fn create_block(
        &mut self,
        data: impl ToString,
        instant: Instant,
        block_data_channel_sender: Sender<Block>,
    ) {
        let new_block = Block::new(
            self.chain.len() as u64,
            data.to_string(),
            self.chain[&self.chain.len() - 1].hash.clone(),
            instant,
            block_data_channel_sender,
        )
        .mine(self.difficulty);
        self.chain.push(new_block);
    }

    pub fn add_block(&mut self, mut block: Block) {
        block.mine(self.difficulty);
        self.chain.push(block);
    }
}
