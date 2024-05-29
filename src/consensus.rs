use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{blockchain::BlockChain, BlockReceiver};

pub async fn accept_agreement(
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
    consensus_data_channels: Arc<Mutex<Vec<BlockReceiver>>>,
) {
    let mut received_blocks = vec![];
    loop {
        //notify consensus
        for channel in consensus_data_channels.lock().await.iter_mut() {
            match channel.block_receiver.try_recv() {
                Ok(block) => {
                    received_blocks.push(block);
                }
                Err(_) => {}
            }
        }

        if received_blocks.len() > consensus_data_channels.lock().await.len() / 2 {
            let mut block_hashes: Vec<(String, u128)> = vec![];
            for block in received_blocks.iter() {
                block_hashes.sort();
                match block_hashes.binary_search_by_key(&block.hash, |(hash, _)| hash.to_string()) {
                    Ok(index) => block_hashes[index].1 += 1,
                    Err(_) => block_hashes.push((block.hash.clone(), 1)),
                }
            }

            let mut max_pair = (String::new(), 0);
            for element in block_hashes.iter() {
                if element.1 > max_pair.1 {
                    max_pair.0 = element.0.clone();
                    max_pair.1 = element.1;
                }
            }

            //it's a bit strange, since we add first one that we find.
            //first of what ?
            //you know we can binary search ?
            //03.46 right now.
            for block in received_blocks.iter() {
                if max_pair.0 == block.hash {
                    match blockchain_thread_safe
                        .lock()
                        .await
                        .push_block(block.clone())
                    {
                        Some(successfully_added_block) => {
                            println!("{:#?}", successfully_added_block);
                            todo!("Notify Whole Network, Reward First Founder or Else")
                        }
                        None => todo!(),
                    }
                }
            }
        }
    }
}
