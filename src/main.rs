use std::sync::Arc;

use rust_blockchain::{
    blockchain::BlockChain,
    client_network, server_network,
    utils::{read_client_config, read_server_config, take_args},
    Runner,
};
use tokio::sync::{broadcast, Mutex};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    match take_args() {
        Some(runner) => match runner {
            Runner::Server => server().await,
            Runner::Client => client().await,
        },
        None => return,
    };
    //todo!("Limbo Block: Not in chain, but processing by others or none. Sync it also");
    //todo!("Consensus should be notified for new block, should forget old blocks");
}

async fn server() {
    let server_config = match read_server_config() {
        Some(server_config) => server_config,
        None => return,
    };

    let blockchain = BlockChain::new(server_config.difficulty.into());
    let blockchain_thread_safe = Arc::new(Mutex::new(blockchain));

    let block_data_channel_sender = broadcast::channel(1).0;

    server_network::start_network(
        server_config,
        blockchain_thread_safe,
        block_data_channel_sender.subscribe(),
    )
    .await;
}

async fn client() {
    let client_config = match read_client_config() {
        Some(client_config) => client_config,
        None => return,
    };
    client_network::start_network(client_config).await;
}
