use std::net::SocketAddr;

use bytes::{Buf, ByteBuf};
use mio::Token;

pub struct Envelope<C: Send> {
    pub address: Option<SocketAddr>,
    pub tok: Token,
    pub msg: C,
}

impl Clone for Envelope<ByteBuf> {
    fn clone(&self) -> Self {
        Envelope {
            address: self.address,
            tok: self.tok,
            msg: ByteBuf::from_slice(self.msg.bytes()),
        }
    }
}
