use traits::Codec;

pub struct CodecStack<In, Mid, Out> {
    left: Box<Codec<In, Mid>>,
    right: Box<Codec<Mid, Out>>,
}

impl <In, Mid, Out> Codec<In, Out> for CodecStack<In, Mid, Out> {
    fn decode(&mut self, buf: &mut In) -> Vec<Out> {
        self.left.decode(buf).iter_mut().flat_map( |mut d|
            self.right.decode(&mut d)).collect()
    }

    fn encode(&self, out: Out) -> In {
        self.left.encode(self.right.encode(out))
    }
}
