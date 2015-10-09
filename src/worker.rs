use std::io::{Error, ErrorKind};
use std::io;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use bytes::{alloc, Buf, ByteBuf};
use mio;
use mio::{EventLoop, EventSet, PollOpt, Handler, Token, TryWrite, TryRead};
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;

use ::{Conn, Receive, Envelope, Codec, ClientInfo};

pub struct Worker<Req: 'static + Send, Res: 'static + Send> {
    pub conns: Slab<Conn<Req, Res>>,
    pub req_codec_factory: fn() -> Box<Codec<ByteBuf, Req>>,
    pub res_codec: Box<Codec<ByteBuf, Res>>,
    pub receiver: Box<(Receive<Req, Res> + Send)>,
}

impl<Req: 'static + Send, Res: 'static + Send> Worker<Req, Res> {
    pub fn conn_readable(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
        tok: Token,
    ) -> io::Result<(ClientInfo, Vec<Req>)> {

        if !self.conns.contains(tok) {
            return Err(Error::new(ErrorKind::Other, "non-existent token"))
        }

        let conn = self.conn(tok);
        let client_info = ClientInfo {
            addr: try!(conn.sock.peer_addr()),
        };
        Ok((client_info, try!(conn.readable(event_loop))))
    }

    pub fn conn_writable(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
        tok: Token,
    ) -> io::Result<()> {
        if !self.conns.contains(tok) {
            return Ok(());
        }

        match self.conn(tok).writable(event_loop) {
            Err(e) => {
                Err(e)
            },
            w => w,
        }
    }

    fn conn<'b>(&'b mut self, tok: Token) -> &'b mut Conn<Req, Res> {
        &mut self.conns[tok]
    }

    pub fn register(
        &mut self,
        sock: TcpStream,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
    ) -> io::Result<Token> {

        let conn = Conn::new(sock, (self.req_codec_factory)());
        self.conns.insert(conn).map(|tok| {
            // Register the connection
            self.conns[tok].token = Some(tok);
            event_loop.register_opt(
                &self.conns[tok].sock,
                tok,
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot()
            ).ok().expect("could not register socket with event loop");
            tok
        }).or_else(|e| Err(Error::new(ErrorKind::Other,
                                      "All connection slots full.")))
    }
}

impl<Req: 'static + Send, Res: 'static + Send> Handler for Worker<Req, Res> {
    type Timeout = ();
    type Message = TcpStream;

    fn ready(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
        token: Token,
        events: EventSet,
    ) {
        if events.is_hup() || events.is_error() {
            self.conns.remove(token);
        }
        if events.is_readable() {
            self.conn_readable(event_loop, token).map(|(client_info, reqs)| {
                for req in reqs.iter() {
                    self.receiver.receive(client_info.clone(), req).map( |res| {
                        let serialized_res = self.res_codec.encode(res);
                        self.conn(token).reply(event_loop, serialized_res);
                    });
                }
            });
        }
        if events.is_writable() {
            self.conn_writable(event_loop, token);
        }
    }

    fn notify(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
        mut sock: TcpStream,
    ) {
        self.register(sock, event_loop);
    }
}
