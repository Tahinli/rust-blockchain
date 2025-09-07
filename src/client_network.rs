use futures_util::{stream::SplitStream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::ClientConfig;

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
    let mut ws_stream = match receive_blockchain(ws_stream_receiver).await {
        Some(ws_stream) => ws_stream,
        None => return,
    };
    loop {
        ws_stream = match receive_block(ws_stream).await {
            Some(ws_stream) => ws_stream,
            None => return,
        }
    }
}

async fn receive_blockchain(mut ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) -> Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    match ws_stream_receiver.next().await {
        Some(message) => match message {
            Ok(message) => {
                println!("{}", message);
                Some(ws_stream_receiver)
            },
            Err(_) => return None,
        },
        None => return None,
    }
}

async fn receive_block(mut ws_stream_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) -> Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>> {
    match ws_stream_receiver.next().await {
        Some(message) => match message {
            Ok(message) => {
                println!("{}", message);
                Some(ws_stream_receiver)
            },
            Err(_) => return None,
        },
        None => return None,
    }
}
