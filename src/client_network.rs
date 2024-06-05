use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

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
    let (ws_stream_sender, ws_stream_receiver) = ws_stream.0.split();
    sync(ws_stream_sender, ws_stream_receiver).await;
}

async fn sync(
    mut ws_stream_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    let (mut ws_stream_receiver, mut blockchain) =
        match receive_blockchain(ws_stream_receiver).await {
            Some((ws_stream_receiver, blockchain)) => (ws_stream_receiver, blockchain),
            None => return,
        };
    loop {
        let block: Block;
        (ws_stream_receiver, block) = match receive_block(ws_stream_receiver).await {
            Some((ws_stream_receiver, block)) => (ws_stream_receiver, block),
            None => return,
        };
        if block.hash == String::new() {
            let block = blockchain.add_block(block);
            ws_stream_sender = match send_block(ws_stream_sender, block).await {
                Some(ws_stream_sender) => ws_stream_sender,
                None => return,
            }
        } else {
            blockchain.push_block(block);
        }
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

async fn send_block(
    mut ws_stream_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    block: Block,
) -> Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>> {
    let block_data = serde_json::json!(block).to_string();
    match ws_stream_sender.send(block_data.into()).await {
        Ok(_) => match ws_stream_sender.flush().await {
            Ok(_) => Some(ws_stream_sender),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
