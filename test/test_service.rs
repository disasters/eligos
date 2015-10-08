use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bytes::{Buf, ByteBuf};

use eligos::{Service, Codec, Framed, Receive, usize_to_array};

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
    counter: Arc<AtomicUsize>,
}

// take requests of usize, return responses of usize
impl Receive<usize, usize> for ByteAddReceiver {
    fn receive(&self, req_bytes: &usize) -> Option<usize> {
        let count = self.counter.fetch_add(*req_bytes, Ordering::SeqCst);
        println!("in handler for srv! bytes so far: {}", count);
        Some(count)
    }
}

#[test]
fn it_works() {
    let receiver = Box::new(ByteAddReceiver {
        counter: Arc::new(AtomicUsize::new(0)),
    });

    let mut service = Service::new(
        6666,
        build_codec, // request codec
        build_codec, // response codec
    ).unwrap();

    let n_workers = 4;
    service.run(vec![receiver]);
}
