use std::cmp;
use std::io;
use std::net;

use consts::*;
use error;
use net::{ReadExt, WriteExt};
use os;

use super::consts::*;

pub fn port(alive: &str) -> Result<(u16,u16), error::Error> {

    let len = alive.len();

    if len > EI_MAXALIVELEN {
        return Err(from_raw_os_error!(ERANGE));
    }

    let mut stream = try!(net::TcpStream::connect(("127.0.0.1", os::getenv("ERL_EPMD_PORT", EPMD_PORT))));

    {
        let mut buf = Vec::with_capacity(3 + len);

        try!(buf.write_u16((1 + len) as u16));
        try!(buf.write_u8(EI_EPMD_PORT2_REQ));
        try!(buf.write_slice(alive.as_bytes()));

        try!(stream.write_slice(buf.as_slice()));
    }

    {
        let mut cursor = io::Cursor::new(try!(stream.read_vec(2)));

        if cursor.read_u8().unwrap() != EI_EPMD_PORT2_RESP {
            return Err(from_raw_os_error!(EIO));
        }

        if cursor.read_i8().unwrap() != EI_SUCCESS {
            return Err(from_raw_os_error!(EINVAL));
        }
    }
    {
        let mut cursor = io::Cursor::new(try!(stream.read_vec(8)));

        let port = cursor.read_u16().unwrap();

        if cursor.read_u8().unwrap() != EI_HIDDEN_NODE {
            return Err(from_raw_os_error!(EIO));
        }

        if cursor.read_u8().unwrap() != EI_MYPROTO {
            return Err(from_raw_os_error!(EIO));
        }

        let dist_high = cursor.read_u16().unwrap();
        let dist_low  = cursor.read_u16().unwrap();

        if dist_low > EI_DIST_HIGH || dist_high < EI_DIST_LOW {
            return Err(from_raw_os_error!(EIO));
        }

        Ok((port, cmp::min(EI_DIST_HIGH, dist_high)))
    }
}
