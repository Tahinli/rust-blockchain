use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::broadcast::Receiver,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

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
        if let Ok(connection) = listener_socket.accept().await {
            let ws_stream = match accept_async(connection.0).await {
                Ok(ws_stream) => ws_stream,
                Err(_) => return ,
            };
            let (ws_stream_sender, _) = ws_stream.split();
            tokio::spawn(sync(
                ws_stream_sender,
                blockchain.clone(),
                block_data_channel_receiver.resubscribe(),
            ));
        }
    }
}

async fn sync(
    ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
    blockchain: BlockChain,
    block_data_channel_receiver: Receiver<Block>,
) {
    let ws_stream_sender = match send_blockchain(ws_stream_sender, blockchain).await {
        Some(ws_stream_sender) => ws_stream_sender,
        None => return,
    };
    send_blocks(ws_stream_sender, block_data_channel_receiver).await;
}

async fn send_blockchain(mut ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>, blockchain: BlockChain) -> Option<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>> {
    let blockchain_data = serde_json::json!({
        "blockchain": blockchain
    })
    .to_string();
    match ws_stream_sender.send(blockchain_data.into()).await {
        Ok(_) => {
            match ws_stream_sender.flush().await {
                Ok(_) => Some(ws_stream_sender),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

async fn send_blocks(mut ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>, mut block_data_channel_receiver: Receiver<Block>) {
    loop {
        match block_data_channel_receiver.recv().await {
            Ok(block) => {
                let block_data = serde_json::json!({
                    "block": block
                })
                .to_string();
                match ws_stream_sender.send(block_data.into()).await {
                    Ok(_) => {
                        if ws_stream_sender.flush().await.is_err() {
                            return;
                        }
                    }
                    Err(_) => return,
                }
            }
            Err(_) => return,
        }
    }
}
