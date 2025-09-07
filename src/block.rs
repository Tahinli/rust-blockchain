use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};

use crate::blockchain::BlockChain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub proof_of_work: u64,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn calculate_hash(&self) -> String {
        let serialized_block_data = serde_json::to_string(&self).unwrap();

        let mut hasher = Sha3_512::new();
        hasher.update(serialized_block_data);
        let hash = hasher.finalize();

        format!("{:x}", hash)
    }

    pub fn new(index: u64, previous_hash: String) -> Self {
        Block {
            index,
            timestamp: Utc::now().timestamp_millis() as u64,
            proof_of_work: 0_u64,
            previous_hash,
            hash: String::new(),
        }
    }

    pub fn mine(&mut self, blockhain: BlockChain) -> Self {
        let mut hash = self.calculate_hash();
        loop {
            if !hash.starts_with(&"0".repeat(blockhain.difficulty)) {
                self.proof_of_work += 1;
                hash = self.calculate_hash();
            } else {
                self.hash = hash;
                break;
            }
        }
        self.clone()
    }
}
