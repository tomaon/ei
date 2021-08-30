#[macro_use]
mod macros;

mod consts;
mod i27;

mod error;
pub use crate::error::Error;

mod term;
pub use crate::term::{Atom, Pid, Port, Ref, Trace};

mod io;
pub use crate::io::{Reader, Writer};

mod de;
pub use crate::de::{from_reader, Deserializer};

mod ser;
pub use crate::ser::{to_vec, Serializer};

pub mod port;
