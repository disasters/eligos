# eligos
Service framework for Rust!

Concepts:
* `Codec` is a state machine that deserializes requests and serializes responses.  There is a default `Framed` codec that just prefixes a message with 4 bytes representing its size.
* `Service` accepts from a listening socket and hands the connection to one of the supplied `Receiver`s
* `Receiver` uses user-supplied logic to handle a request, and optionally respond.  Every Receiver has its own thread and MIO event loop.

### Running
###### Cargo.toml
```rust
[dependencies]
eligos = "0.1.0"
bytes = "0.2.11"
```
###### Code
```rust
extern crate bytes;
extern crate eligos;

use self::bytes::{Buf, ByteBuf};
use self::eligos::{Service, Codec, Receive, ClientInfo};

// This codec just produces the length of the bytes you give it.
// On the way out, it takes a usize and turns it into a string.
struct CountCodec;

// Codecs are state machines that incrementally eat bytes, and
// returns the requests that have been parsed so far.
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

// Services will create a codec for each connection.  This tells them how!
fn build_codec() -> Box<Codec<ByteBuf, usize>> {
    Box::new(CountCodec)
}

// Receivers get requests that are returned by Codecs.
struct ByteAddReceiver {
    counter: usize,
}

// Take requests of usize, return responses of usize.  We can use different
// Codecs for the request and response types.
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
