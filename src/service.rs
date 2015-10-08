use std::io::{Error, ErrorKind};
use std::io;
use std::sync::Arc;
use std::thread;

use bytes::{alloc, Buf, ByteBuf};
use mio::{EventLoop, EventSet, PollOpt, Handler, Sender, Token, TryWrite, TryRead};
use mio::tcp::{TcpListener, TcpSocket, TcpStream};
use mio::util::Slab;

use ::{Receive, Worker, Envelope, Codec};

pub struct Service<Req: 'static + Send, Res: 'static + Send> {
    workers: Vec<Sender<TcpStream>>,
    sock: TcpListener,
    req_codec_factory: fn() -> Box<Codec<ByteBuf, Req>>,
    res_codec_factory: fn() -> Box<Codec<ByteBuf, Res>>,
}

impl<Req: 'static + Send, Res: 'static + Send> Service<Req, Res> {
    pub fn new(
        accept_port: u16,
        req_codec_factory: fn() -> Box<Codec<ByteBuf, Req>>,
        res_codec_factory: fn() -> Box<Codec<ByteBuf, Res>>,
    ) -> io::Result<Service<Req, Res>> {

        let accept_addr = format!("0.0.0.0:{}", accept_port).parse().unwrap();
        println!("binding to {} for connections", accept_addr);
        let sock = try!(TcpListener::bind(&accept_addr));

        Ok(Service {
            sock: sock,
            workers: vec![],
            req_codec_factory: req_codec_factory,
            res_codec_factory: res_codec_factory,
        })
    }

    pub fn run<'a>(
        &mut self,
        receivers: Vec<Box<(Receive<Req, Res> + Send)>>,
    ) {
        let mut workers = vec![];
        for receiver in receivers {
            // Each receiver gets its own event loop and thread to run it
            let mut worker_loop: EventLoop<Worker<Req, Res>> = 
                EventLoop::new().unwrap();

            // The worker_loop channel is used to receive new connections
            // from the single acceptor thread.
            workers.push(worker_loop.channel());
            let req_codec_clone = self.req_codec_factory.clone();
            let res_codec_clone = self.res_codec_factory.clone();

            thread::Builder::new().name(format!("worker loop")).spawn( move || {
                let res_codec = (res_codec_clone)();
                let max_conns = 131072;
                let mut worker = Worker {
                    conns: Slab::new_starting_at(Token(0), max_conns),
                    req_codec_factory: req_codec_clone,
                    res_codec: res_codec,
                    receiver: receiver,
                };
                worker_loop.run(&mut worker).unwrap();
            });
        }

        self.workers = workers;

        // We use a single acceptor thread that does nothing but accept
        // and hand off to a worker event loop.
        let mut accept_loop: EventLoop<Service<Req, Res>> = 
            EventLoop::new().unwrap();

        accept_loop.register_opt(
            &self.sock,
            Token(0),
            EventSet::readable(),
            PollOpt::edge(), // TODO(tyler) measure if we need to use oneshot
        ).unwrap();

        accept_loop.run(self).unwrap();
    }

    pub fn accept(
        &mut self,
    ) -> io::Result<()> {
        let sock = try!(self.sock.accept());
        // TODO(tyler) load balancing logic here
        if sock.is_some() {
            self.workers[0].send(sock.unwrap());
        }
        Ok(())
    }
}

impl<Req: 'static + Send, Res: 'static + Send> Handler for Service<Req, Res> {
    type Timeout = ();
    type Message = ();

    fn ready(
        &mut self,
        event_loop: &mut EventLoop<Service<Req, Res>>,
        token: Token,
        events: EventSet,
    ) {
        self.accept();
    }
}
