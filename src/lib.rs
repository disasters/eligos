#![crate_id = "eligos"]
#![crate_type = "lib"]

extern crate mio;
extern crate bytes;

mod envelope;
mod codec;
mod worker;
mod conn;
mod service;
mod receive;

pub use worker::Worker;
pub use conn::Conn;
pub use envelope::Envelope;
pub use codec::{Codec, Framed, usize_to_array, array_to_usize};
pub use service::Service;
pub use receive::Receive;

pub mod traits {
    //! All traits are re-exported here to allow glob imports.
    pub use {Codec, Receive};
}
