use rust_blockchain::{
    blockchain::BlockChain,
    network::start_network,
    utils::{read_server_config, take_args},
    Runner,
};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    match take_args() {
        Some(runner) => match runner {
            Runner::Server => server().await,
            Runner::Client => todo!(),
        },
        None => return,
    };
}

async fn server() {
    let server_config = match read_server_config() {
        Some(server_config) => server_config,
        None => return,
    };

    let blockchain = BlockChain::new(server_config.difficulty.into());
    let block_data_channel_sender = broadcast::channel(1).0;
    start_network(
        server_config,
        &blockchain,
        block_data_channel_sender.subscribe(),
    )
    .await;
}
