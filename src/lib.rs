use std::net::IpAddr;

use block::Block;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

pub mod block;
pub mod blockchain;
pub mod client_network;
pub mod consensus;
pub mod server_network;
mod test;
pub mod utils;

pub enum Runner {
    Server,
    Client,
}
pub struct ServerConfig {
    pub server_address: IpAddr,
    pub port: u16,
    pub difficulty: u8,
}
pub struct ClientConfig {
    pub server_address: IpAddr,
    pub port: u16,
}

#[derive(Debug)]
pub struct BlockReceiver {
    pub block_receiver: Receiver<Block>,
    pub uuid: Uuid,
}

impl Ord for BlockReceiver {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl PartialOrd for BlockReceiver {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BlockReceiver {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for BlockReceiver {}
