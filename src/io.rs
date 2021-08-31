use std::io;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

use crate::error::Error;

fn into(vec: Vec<u8>) -> Result<String, Error> {
    String::from_utf8(vec).map_err(Error::String)
}

#[derive(Debug)]
pub enum Number {
    U8(u8),
    I32(i32),
    SmallBig(u64, u8),
}

pub struct Reader<R> {
    r: R,
}

impl<R> Reader<R>
where
    R: io::Read,
{
    #[inline]
    pub fn new(r: R) -> Self
    where
        R: io::Read,
    {
        Reader { r }
    }

    #[inline]
    pub fn read_exact_const<const LEN: usize>(&mut self) -> Result<[u8; LEN], Error> {
        let mut buf = [0u8; LEN];
        self.read_exact(&mut buf).and_then(|()| Ok(buf))
    }

    #[inline]
    pub fn read_exact_u8(&mut self) -> Result<Vec<u8>, Error> {
        self.read_u8()
            .and_then(|u| self.read_exact_usize(u as usize))
    }

    #[inline]
    pub fn read_exact_u16(&mut self) -> Result<Vec<u8>, Error> {
        self.read_u16()
            .and_then(|u| self.read_exact_usize(u as usize))
    }

    #[inline]
    pub fn read_exact_u32(&mut self) -> Result<Vec<u8>, Error> {
        self.read_u32()
            .and_then(|u| self.read_exact_usize(u as usize))
    }

    #[inline]
    pub fn read_exact_usize(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut vec = vec![0u8; len];
        self.read_exact(&mut vec).and_then(|()| Ok(vec))
    }

    #[inline]
    pub fn read_string_u8(&mut self) -> Result<String, Error> {
        self.read_exact_u8().and_then(|v| into(v))
    }

    #[inline]
    pub fn read_string_u16(&mut self) -> Result<String, Error> {
        self.read_exact_u16().and_then(|v| into(v))
    }

    // #region std::io

    #[inline]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.r.read_exact(buf).map_err(Error::Io)
    }

    // #endregion

    // #region byteorder

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16, Error> {
        self.r.read_i16::<NetworkEndian>().map_err(Error::Io)
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32, Error> {
        self.r.read_i32::<NetworkEndian>().map_err(Error::Io)
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.r.read_u8().map_err(Error::Io)
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16, Error> {
        self.r.read_u16::<NetworkEndian>().map_err(Error::Io)
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.r.read_u32::<NetworkEndian>().map_err(Error::Io)
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64, Error> {
        self.r.read_u64::<NetworkEndian>().map_err(Error::Io)
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64, Error> {
        self.r.read_f64::<NetworkEndian>().map_err(Error::Io)
    }

    // #endregion
}

pub struct Writer<W> {
    w: W,
}

impl<W> Writer<W>
where
    W: io::Write,
{
    #[inline]
    pub fn new(w: W) -> Self
    where
        W: io::Write,
    {
        Writer { w }
    }

    // #region std::io

    #[inline]
    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.w.write_all(buf).map_err(Error::Io)
    }

    #[inline]
    pub fn flush(&mut self) -> Result<(), Error> {
        self.w.flush().map_err(Error::Io)
    }

    // #endregion

    // #region byteorder

    #[inline]
    pub fn write_i16(&mut self, i: i16) -> Result<(), Error> {
        self.w.write_i16::<NetworkEndian>(i).map_err(Error::Io)
    }

    #[inline]
    pub fn write_i32(&mut self, i: i32) -> Result<(), Error> {
        self.w.write_i32::<NetworkEndian>(i).map_err(Error::Io)
    }

    #[inline]
    pub fn write_u8(&mut self, u: u8) -> Result<(), Error> {
        self.w.write_u8(u).map_err(Error::Io)
    }

    #[inline]
    pub fn write_u16(&mut self, u: u16) -> Result<(), Error> {
        self.w.write_u16::<NetworkEndian>(u).map_err(Error::Io)
    }

    #[inline]
    pub fn write_u32(&mut self, u: u32) -> Result<(), Error> {
        self.w.write_u32::<NetworkEndian>(u).map_err(Error::Io)
    }

    #[inline]
    pub fn write_u64(&mut self, u: u64) -> Result<(), Error> {
        self.w.write_u64::<NetworkEndian>(u).map_err(Error::Io)
    }

    #[inline]
    pub fn write_f64(&mut self, f: f64) -> Result<(), Error> {
        self.w.write_f64::<NetworkEndian>(f).map_err(Error::Io)
    }

    // #endregion
}
