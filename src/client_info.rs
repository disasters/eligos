use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub addr: SocketAddr,
}
