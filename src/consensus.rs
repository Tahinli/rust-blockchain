use std::sync::Arc;

use tokio::sync::{broadcast::Sender, Mutex};

use crate::{block::Block, blockchain::BlockChain, BlockReceiver};

pub async fn accept_agreement(
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
    consensus_data_channels: Arc<Mutex<Vec<BlockReceiver>>>,
    block_data_channel_sender: Sender<Block>,
    limbo_block: Arc<Mutex<Block>>,
) {
    let mut received_blocks = vec![];
    loop {
        for channel in consensus_data_channels.lock().await.iter_mut() {
            match channel.block_receiver.try_recv() {
                Ok(block) => {
                    if block.previous_hash == limbo_block.lock().await.previous_hash {
                        received_blocks.push(block);
                    }
                }
                Err(_) => {}
            }
        }

        if received_blocks.len() > consensus_data_channels.lock().await.len() / 2 {
            let mut block_hashes: Vec<(String, u128)> = vec![];
            for block in received_blocks.iter() {
                block_hashes.sort_by_key(|(hash, _counter)| hash.to_string());
                match block_hashes.binary_search_by_key(&block.hash, |(hash, _)| hash.to_string()) {
                    Ok(index) => block_hashes[index].1 += 1,
                    Err(_) => block_hashes.push((block.hash.clone(), 1)),
                }
            }

            block_hashes.sort_by_key(|(_hash, counter)| *counter);
            let max_pair = block_hashes[0].clone();

            for block in received_blocks.iter() {
                if max_pair.0 == block.hash {
                    match blockchain_thread_safe
                        .lock()
                        .await
                        .push_block(block.clone())
                    {
                        Some(successfully_added_block) => {
                            match block_data_channel_sender.send(successfully_added_block) {
                                Ok(_) => {
                                    *limbo_block.lock().await =
                                        blockchain_thread_safe.lock().await.genesis_block.clone()
                                }
                                Err(_) => {}
                            }
                        }
                        None => todo!(),
                    }
                }
            }
        }
    }
}
