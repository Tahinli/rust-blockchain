use std::net::IpAddr;

pub mod block;
pub mod blockchain;
pub mod client_network;
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
