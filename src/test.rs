#[cfg(test)]
use crate::blockchain::BlockChain;
#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
use std::time::Instant;
#[cfg(test)]
use tokio::sync::broadcast::channel;

#[tokio::test]
async fn create_blockchain() {
    let blockchain = BlockChain::new(1);
    assert_eq!(blockchain.difficulty, 1);
    assert_eq!(blockchain.genesis_block.data, "Tahinli");
    assert_eq!(blockchain.genesis_block.hash, "");
    assert_eq!(blockchain.genesis_block.index, 0);
    assert_eq!(blockchain.genesis_block.previous_hash, "");
    assert_eq!(blockchain.genesis_block.proof_of_work, 0);
    assert_eq!(
        blockchain.genesis_block.hash_time_cost,
        Duration::from_secs(0)
    );
    assert_eq!(blockchain.chain.len(), 1);
    assert_eq!(blockchain.chain[0].data, "Tahinli");
    assert_eq!(blockchain.chain[0].hash, "");
    assert_eq!(blockchain.chain[0].index, 0);
    assert_eq!(blockchain.chain[0].previous_hash, "");
    assert_eq!(blockchain.chain[0].proof_of_work, 0);
    assert_eq!(blockchain.chain[0].hash_time_cost, Duration::from_secs(0));
}
#[tokio::test]
async fn create_block() {
    let instant = Instant::now();
    let mut blockchain = BlockChain::new(1);
    let block_data_channel_sender = channel(1).0;
    BlockChain::add_block(
        &mut blockchain,
        "Ahmet Kaan Gümüş".to_string(),
        instant,
        block_data_channel_sender.clone(),
    );
    assert_eq!(blockchain.chain[0].data, "Tahinli");
    assert_eq!(blockchain.chain[0].hash, "");
    assert_eq!(blockchain.chain[0].index, 0);
    assert_eq!(blockchain.chain[0].previous_hash, "");
    assert_eq!(blockchain.chain[0].proof_of_work, 0);
    assert_eq!(blockchain.chain[0].hash_time_cost, Duration::from_secs(0));
    assert_eq!(blockchain.chain[1].data, "Ahmet Kaan Gümüş");
    assert_eq!(blockchain.chain[1].previous_hash, "");
    assert_eq!(blockchain.chain[1].index, 1);
    assert_eq!(blockchain.chain.len(), 2);
}
