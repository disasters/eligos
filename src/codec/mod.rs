mod framed;
mod codec_stack;

pub use self::framed::{Framed, usize_to_array, array_to_usize};

pub trait Codec<In: ?Sized, Out: Sized> {
    fn decode(&mut self, buf: &mut In) -> Vec<Out>;
    fn encode(&self, a: Out) -> In;
}
