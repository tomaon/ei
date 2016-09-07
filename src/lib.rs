pub use connect::accept;
pub use decoder::decode;
pub use encoder::encode;
pub use epmd::{port, publish};
pub use error::Error;
pub use ports::{recv, send};
pub use term::{Atom, Pid, Port, Ref, Trace};

pub use handle::handle;

#[macro_use]
mod error;
#[macro_use]
mod macros;

mod connect;
mod consts;
mod convert;
mod decoder;
mod encoder;
mod epmd;
mod fs;
mod i27;
mod md5;
mod net;
mod os;
mod ports;
mod term;

mod handle;

extern crate crypto;
extern crate rustc_serialize;
