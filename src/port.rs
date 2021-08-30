use std::io;

use crate::consts::*;
use crate::error::Error;
use crate::io::{Reader, Writer};

impl<R> Reader<R>
where
    R: io::Read,
{
    pub fn recv(&mut self) -> Result<Vec<u8>, Error> {
        let u = self.read_u16()?;

        if self.read_u8()? != ERL_VERSION_MAGIC {
            return Err(invalid_data!("ERL_VERSION_MAGIC"));
        }

        self.read_exact_usize(u as usize - 1)
    }
}

impl<W> Writer<W>
where
    W: io::Write,
{
    pub fn send(&mut self, v: &[u8]) -> Result<(), Error> {
        self.write_u16(v.len() as u16 + 1)?;
        self.write_u8(ERL_VERSION_MAGIC)?;
        self.write_all(v)?;
        self.flush()
    }
}
