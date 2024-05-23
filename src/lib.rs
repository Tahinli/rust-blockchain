use std::net::IpAddr;

pub mod block;
pub mod blockchain;
pub mod network;
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
