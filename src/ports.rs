use std::io::{self, Write};

use consts::*;
use error;
use net::{ReadExt, WriteExt};

pub fn recv(r: &mut io::Read) -> Result<io::Cursor<Vec<u8>>, error::Error> {
    match try!(r.read_u16()) {
        0 => Err(from_raw_os_error!(EINTR)),
        u => {

            let mut cursor = io::Cursor::new(try!(r.read_vec(u as usize)));

            if cursor.read_u8().unwrap() != ERL_VERSION_MAGIC {
                return Err(from_raw_os_error!(EIO))
            }

            Ok(cursor)
        },
    }
}

pub fn send(w: &mut io::Write, v: &[u8]) -> Result<(), error::Error> {
    let len = 1 + v.len();
    let mut writer = io::BufWriter::with_capacity(2 + len, w);
    writer.write_u16(len as u16).unwrap();
    writer.write_u8(ERL_VERSION_MAGIC).unwrap();
    writer.write_slice(v).unwrap();
    writer.flush().or_else(|e| Err(From::from(e)))
}
