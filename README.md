# eligos
Service framework for Rust!

Concepts:
* `Codec` deserializes requests and serialize responses
* `Service` accepts from a listening socket and hands the connection to one of the supplied `Receiver`s
* `Receiver` uses user-supplied logic to handle a request and optionally respond.  Every Receiver has its own thread and MIO event loop.

```
extern crate bytes;
extern crate eligos;

use self::bytes::{Buf, ByteBuf};
use self::eligos::{Service, Codec, Receive};

struct CountCodec;

impl Codec<ByteBuf, usize> for CountCodec {
    fn decode(&mut self, buf: &mut ByteBuf) -> Vec<usize> {
        println!("got some bytes in codec!");
        vec![buf.bytes().len()]
    }

    fn encode(&self, us: usize) -> ByteBuf {
        let byte_str = format!("{}", us);
        let bytes = byte_str.as_bytes();
        ByteBuf::from_slice(bytes)
    }
}

fn build_codec() -> Box<Codec<ByteBuf, usize>> {
    Box::new(CountCodec)
}

struct ByteAddReceiver {
    counter: usize,
}

// take requests of usize, return responses of usize
impl Receive<usize, usize> for ByteAddReceiver {
    fn receive(&mut self, client_info: ClientInfo, req_bytes: &usize) -> Option<usize> {
        self.counter += *req_bytes;
        println!("got req from {:?}! bytes so far: {}", client_info, self.counter);
        Some(self.counter)
    }
}

fn main() {
    let receiver = Box::new(ByteAddReceiver {
        counter: 0,
    });

    let mut service = Service::new(
        6666,
        build_codec, // request codec
        build_codec, // response codec
    ).unwrap();

    let n_workers = 4;
    service.run(vec![receiver]);
}
```
