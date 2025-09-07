use futures_util::{stream::SplitStream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::{block::Block, blockchain::BlockChain, ClientConfig};

pub async fn start_network(client_config: ClientConfig) {
    let ws_stream = match connect_async(format!(
        "ws://{}:{}",
        client_config.server_address, client_config.port
    ))
    .await
    {
        Ok(ws_stream) => ws_stream,
        Err(_) => return,
    };
    let (_, ws_receiver) = ws_stream.0.split();
    sync(ws_receiver).await;
}

async fn sync(ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    let (mut ws_stream, mut blockchain) = match receive_blockchain(ws_stream_receiver).await {
        Some((ws_stream, blockchain)) => (ws_stream, blockchain),
        None => return,
    };
    loop {
        let block: Block;
        (ws_stream, block) = match receive_block(ws_stream).await {
            Some((ws_stream, block)) => (ws_stream, block),
            None => return,
        };
        blockchain.add_block(block);
    }
}

async fn receive_blockchain(
    mut ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Option<(
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    BlockChain,
)> {
    match ws_stream_receiver.next().await {
        Some(message) => match message {
            Ok(message) => {
                if let tokio_tungstenite::tungstenite::Message::Text(message) = message {
                    let blockchain: BlockChain = match serde_json::from_str(&message[..]) {
                        Ok(blockchain) => blockchain,
                        Err(_) => return None,
                    };
                    Some((ws_stream_receiver, blockchain))
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        },
        None => return None,
    }
}

async fn receive_block(
    mut ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Option<(
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    Block,
)> {
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
                    return None;
                }
            }
            Err(_) => return None,
        },
        None => return None,
    }
}
