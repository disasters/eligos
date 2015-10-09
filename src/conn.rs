use std::io;
use std::marker::PhantomData;
use std::sync::mpsc::Sender;

use bytes::{Buf, ByteBuf};
use mio::{EventLoop, EventSet, PollOpt, Token, TryWrite, TryRead};
use mio::tcp::TcpStream;

use ::{Worker, Envelope, Codec};

pub struct Conn<Req: 'static + Send, Res: 'static + Send> {
    pub sock: TcpStream,
    pub res_bufs: Vec<ByteBuf>, // TODO(tyler) use proper dequeue
    pub res_remaining: usize,
    pub codec: Box<Codec<ByteBuf, Req>>,
    pub token: Option<Token>,
    pub interest: EventSet,
    _res: PhantomData<Res>,
}

impl<Req: 'static + Send, Res: 'static + Send> Conn<Req, Res> {
    pub fn new(
        sock: TcpStream,
        codec: Box<Codec<ByteBuf, Req>>
    ) -> Conn<Req, Res> {
        Conn {
            sock: sock,
            codec: codec,
            res_bufs: vec![],
            res_remaining: 0,
            token: None,
            interest: EventSet::hup(),
            _res: PhantomData,
        }
    }

    pub fn writable(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>
    ) -> io::Result<()> {
        if self.res_bufs.len() == 0 {
            // no responses yet, don't reregister
            return Ok(())
        }
        let mut res_buf = self.res_bufs.remove(0);

        match self.sock.try_write_buf(&mut res_buf) {
            Ok(None) => {
                self.interest.insert(EventSet::writable());
            }
            Ok(Some(r)) => {
                self.res_remaining -= r;
                if self.res_remaining == 0 {
                    // we've written the whole response, now let's wait to read
                    self.interest.insert(EventSet::readable());
                    self.interest.remove(EventSet::writable());
                }
            }
            Err(e) => {
                match e.raw_os_error() {
                    Some(32) => {
                        // client disconnected
                    },
                    Some(e) =>
                        println!("not implemented; client os err={:?}", e),
                    _ =>
                        println!("not implemented; client err={:?}", e),
                };
                // Don't reregister.
                return Err(e);
            },
        }

        // push res back if it's not finished
        if res_buf.remaining() != 0 {
            self.res_bufs.insert(0, res_buf);
        }

        self.reregister(event_loop);

        Ok(())
    }

    pub fn readable(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>
    ) -> io::Result<Vec<Req>> {

        // TODO(tyler) get rid of this double copying and read
        // directly to codec
        let mut req_buf = ByteBuf::mut_with_capacity(1024);

        match self.sock.try_read_buf(&mut req_buf) {
            Ok(None) => {
                panic!("got readable, but can't read from the socket");
                self.interest.insert(EventSet::readable());
            }
            Ok(Some(r)) => {
                self.interest.insert(EventSet::readable());
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                self.interest.remove(EventSet::readable());
            }
        };

        self.reregister(event_loop);

        Ok(self.codec.decode(&mut req_buf.flip()))
    }

    fn reregister(&mut self, event_loop: &mut EventLoop<Worker<Req, Res>>) {
        event_loop.reregister(
            &self.sock,
            self.token.unwrap(),
            self.interest,
            PollOpt::edge() | PollOpt::oneshot(),
        );
    }

    pub fn reply(
        &mut self,
        event_loop: &mut EventLoop<Worker<Req, Res>>,
        res: ByteBuf
    ) {
        self.res_remaining += res.bytes().len();
        self.res_bufs.push(res);
        self.interest.insert(EventSet::writable());
        self.reregister(event_loop);
    }
}
