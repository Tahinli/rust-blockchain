use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::broadcast::Receiver,
};

use crate::{block::Block, blockchain::BlockChain, ServerConfig};

pub async fn start_network(
    server_config: ServerConfig,
    blockchain: &BlockChain,
    block_data_channel_receiver: Receiver<Block>,
) {
    let listener_socket = match TcpListener::bind(format!(
        "{}:{}",
        server_config.server_address, server_config.port
    ))
    .await
    {
        Ok(listener_socket) => listener_socket,
        Err(_) => return,
    };

    loop {
        match listener_socket.accept().await {
            Ok(connection) => {
                tokio::spawn(sync(
                    connection.0,
                    blockchain.clone(),
                    block_data_channel_receiver.resubscribe(),
                ));
            }
            Err(_) => {}
        }
    }
}

async fn sync(
    tcp_stream: TcpStream,
    blockchain: BlockChain,
    block_data_channel_receiver: Receiver<Block>,
) {
    let tcp_stream = send_blockchain(tcp_stream, blockchain).await;
    send_block(tcp_stream, block_data_channel_receiver).await;
}

async fn send_blockchain(mut tcp_stream: TcpStream, blockchain: BlockChain) -> TcpStream {
    let blockchain_data = serde_json::json!({
        "blockchain": blockchain
    })
    .to_string();
    match tcp_stream.write_all(&blockchain_data.as_bytes()).await {
        Ok(_) => match tcp_stream.flush().await {
            Ok(_) => {}
            Err(_) => {}
        },
        Err(_) => {}
    }
    tcp_stream
}

async fn send_block(mut tcp_stream: TcpStream, mut block_data_channel_receiver: Receiver<Block>) {
    loop {
        match block_data_channel_receiver.recv().await {
            Ok(block) => {
                let block_data = serde_json::json!({
                    "block": block
                })
                .to_string();
                match tcp_stream.write_all(&block_data.as_bytes()).await {
                    Ok(_) => match tcp_stream.flush().await {
                        Ok(_) => {}
                        Err(_) => {}
                    },
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }
}
