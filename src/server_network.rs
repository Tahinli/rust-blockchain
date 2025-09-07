use std::sync::Arc;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{broadcast::Receiver, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

use crate::{block::Block, blockchain::BlockChain, ServerConfig};

pub async fn start_network(
    server_config: ServerConfig,
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
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
                Err(_) => return,
            };
            let (ws_stream_sender, _) = ws_stream.split();
            tokio::spawn(sync(
                ws_stream_sender,
                blockchain_thread_safe.clone(),
                block_data_channel_receiver.resubscribe(),
            ));
        }
    }
}

async fn sync(
    ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
    mut block_data_channel_receiver: Receiver<Block>,
) {
    let mut ws_stream_sender = match send_blockchain(ws_stream_sender, blockchain_thread_safe).await
    {
        Some(ws_stream_sender) => ws_stream_sender,
        None => return,
    };
    loop {
        let block = match block_data_channel_receiver.recv().await {
            Ok(block) => block,
            Err(_) => return,
        };
        ws_stream_sender = match send_block(ws_stream_sender, block).await {
            Some(ws_stream_sender) => ws_stream_sender,
            None => return,
        }
    }
}

async fn send_blockchain(
    mut ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
) -> Option<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>> {
    let blockchain = blockchain_thread_safe.lock().await;
    let blockchain_data = serde_json::json!(*blockchain).to_string();
    match ws_stream_sender.send(blockchain_data.into()).await {
        Ok(_) => match ws_stream_sender.flush().await {
            Ok(_) => Some(ws_stream_sender),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

async fn send_block(
    mut ws_stream_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
    block: Block,
) -> Option<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>> {
    let block_data = serde_json::json!(block).to_string();
    match ws_stream_sender.send(block_data.into()).await {
        Ok(_) => match ws_stream_sender.flush().await {
            Ok(_) => Some(ws_stream_sender),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
