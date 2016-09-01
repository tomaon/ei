use std::io;
use std::net;

use consts::*;
use error;
use net::{ReadExt, WriteExt};
use os;

use super::consts::*;

pub fn publish(alive: &str, port: u16) -> Result<net::TcpStream, error::Error> {

    let len = alive.len();

    if len > EI_MAXALIVELEN {
        return Err(from_raw_os_error!(ERANGE));
    }

    let mut stream = try!(net::TcpStream::connect(("127.0.0.1", os::getenv("ERL_EPMD_PORT", EPMD_PORT))));

    {
        let mut buf = Vec::with_capacity(15 + len);

        try!(buf.write_u16((13 + len) as u16));
        try!(buf.write_u8(EI_EPMD_ALIVE2_REQ));
        try!(buf.write_u16(port));
        try!(buf.write_u8(EI_HIDDEN_NODE));
        try!(buf.write_u8(EI_MYPROTO));
        try!(buf.write_u16(EI_DIST_HIGH));
        try!(buf.write_u16(EI_DIST_LOW));
        try!(buf.write_u16(len as u16));
        try!(buf.write_slice(alive.as_bytes()));
        try!(buf.write_u16(0));

        try!(stream.write_slice(buf.as_slice()));
    }

    {
        let mut cursor = io::Cursor::new(try!(stream.read_vec(2)));

        if cursor.read_u8().unwrap() != EI_EPMD_ALIVE2_RESP {
            return Err(from_raw_os_error!(EIO));
        }

        if cursor.read_i8().unwrap() != EI_SUCCESS {
            return Err(from_raw_os_error!(EINVAL));
        }
    }
    {
        let mut cursor = io::Cursor::new(try!(stream.read_vec(2)));

        cursor.read_i8().unwrap();
    }

    Ok(stream)
}
