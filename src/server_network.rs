use std::sync::Arc;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex,
    },
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use crate::{block::Block, blockchain::BlockChain, consensus, BlockReceiver, ServerConfig};

pub async fn start_network(
    server_config: ServerConfig,
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
    block_data_channel_sender: Sender<Block>,
    limbo_block: Arc<Mutex<Block>>,
) {
    let block_data_channel_receiver = block_data_channel_sender.subscribe();
    let listener_socket = match TcpListener::bind(format!(
        "{}:{}",
        server_config.server_address, server_config.port
    ))
    .await
    {
        Ok(listener_socket) => listener_socket,
        Err(_) => return,
    };
    let consensus_data_channels = Arc::new(Mutex::new(vec![]));
    tokio::spawn(consensus::accept_agreement(
        blockchain_thread_safe.clone(),
        consensus_data_channels.clone(),
        block_data_channel_sender,
        limbo_block.clone(),
    ));
    loop {
        if let Ok(connection) = listener_socket.accept().await {
            let ws_stream = match accept_async(connection.0).await {
                Ok(ws_stream) => ws_stream,
                Err(_) => continue,
            };
            let (ws_stream_sender, ws_stream_receiver) = ws_stream.split();
            let blockchain_thread_safe = blockchain_thread_safe.clone();
            let block_data_channel_receiver = block_data_channel_receiver.resubscribe();
            let consensus_data_channel_sender = broadcast::channel(1).0;
            let block_receiver = BlockReceiver {
                block_receiver: consensus_data_channel_sender.subscribe(),
                uuid: Uuid::new_v4(),
            };

            let uuid = block_receiver.uuid;

            consensus_data_channels.lock().await.push(block_receiver);

            let consensus_data_channels = consensus_data_channels.clone();
            let limbo_block = limbo_block.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = sync_client(
                        ws_stream_sender,
                        blockchain_thread_safe,
                        block_data_channel_receiver,
                        limbo_block,
                    ) => {
                        let mut consensus_data_channels = consensus_data_channels.lock().await;
                        consensus_data_channels.sort();
                        if let Ok(block_receiver_index) = consensus_data_channels.binary_search_by_key(&uuid, |block_receive| block_receive.uuid) {
                            consensus_data_channels.remove(block_receiver_index);
                        };
                        drop(consensus_data_channels);
                    }

                    _ = sync_server(ws_stream_receiver, consensus_data_channel_sender) => {
                        let mut consensus_data_channels = consensus_data_channels.lock().await;
                        consensus_data_channels.sort();
                        if let Ok(block_receiver_index) = consensus_data_channels.binary_search_by_key(&uuid, |block_receive| block_receive.uuid) {
                            consensus_data_channels.remove(block_receiver_index);
                        };
                        drop(consensus_data_channels);
                    }
                }
            });
        }
    }
}

async fn sync_server(
    mut ws_stream_receiver: SplitStream<WebSocketStream<TcpStream>>,
    consensus_data_channel_sender: Sender<Block>,
) {
    loop {
        let block;
        (ws_stream_receiver, block) = match receive_block(ws_stream_receiver).await {
            Some((ws_stream_receiver, block)) => (ws_stream_receiver, block),
            None => return,
        };
        if consensus_data_channel_sender.send(block).is_err() {
            return;
        }
    }
}

async fn receive_block(
    mut ws_stream_receiver: SplitStream<WebSocketStream<TcpStream>>,
) -> Option<(SplitStream<WebSocketStream<TcpStream>>, Block)> {
    match ws_stream_receiver.next().await {
        Some(message) => match message {
            Ok(message) => {
                if let tokio_tungstenite::tungstenite::Message::Text(message) = message {
                    let block: Block = match serde_json::from_str(&message[..]) {
                        Ok(block) => block,
                        Err(_) => return None,
                    };
                    Some((ws_stream_receiver, block))
                } else {
                    None
                }
            }
            Err(_) => None,
        },
        None => None,
    }
}

async fn sync_client(
    ws_stream_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    blockchain_thread_safe: Arc<Mutex<BlockChain>>,
    mut block_data_channel_receiver: Receiver<Block>,
    limbo_block: Arc<Mutex<Block>>,
) {
    let mut ws_stream_sender =
        match send_blockchain(ws_stream_sender, blockchain_thread_safe.clone()).await {
            Some(ws_stream_sender) => ws_stream_sender,
            None => return,
        };
    let limbo_block = limbo_block.lock().await;
    if limbo_block.timestamp != blockchain_thread_safe.lock().await.genesis_block.timestamp {
        ws_stream_sender = match send_block(ws_stream_sender, limbo_block.clone()).await {
            Some(ws_stream_sender) => ws_stream_sender,
            None => return,
        }
    }
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
    mut ws_stream_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
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
    mut ws_stream_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
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
