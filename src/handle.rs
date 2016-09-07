use std::io;
use std::net;

use connect;
use consts::*;
use decoder;
use encoder;
use error;
use fs;
use net::{ReadExt, WriteExt};
use term;

pub fn handle<F>(stream: &mut net::TcpStream, nodename: &String, mut f: F) -> Result<(), error::Error>
    where F: FnMut(&mut io::Read) -> Result<Vec<u8>, error::Error> {

    if nodename.len() as usize > MAXNODELEN {
        return Err(from_raw_os_error!(ERANGE));
    }

    let cookie = fs::cookie(); // TODO

    if cookie.len() as usize > EI_MAX_COOKIE_SIZE {
        return Err(from_raw_os_error!(ERANGE));
    }

    try!(connect::accept(stream, nodename, &cookie));

    loop {

        let size = try!(stream.read_u32()) as usize; // TODO

        if size > 0 { // tick?

            let mut cursor = io::Cursor::new(try!(stream.read_vec(size)));

            if cursor.read_u8().unwrap() != ERL_PASS_THROUGH {
                return Err(from_raw_os_error!(EIO));
            }

            if cursor.read_u8().unwrap() != ERL_VERSION_MAGIC {
                return Err(from_raw_os_error!(EIO));
            }

            match try!(decoder::decode(&mut cursor)) {

                term::Msg::RegSend { from, .. } => {

                    if cursor.read_u8().unwrap() != ERL_VERSION_MAGIC {
                        return Err(from_raw_os_error!(EIO));
                    }

                    if let Ok(data) = f(&mut cursor) {

                        let mut head = Vec::with_capacity(16 + MAXATOMLEN_UTF8);

                        try!(encoder::encode(&mut head, &term::Msg::Send {
                            cookie: term::Atom::UTF8Small("".to_string()),
                            to: from,
                        }));

                        let len = 2 + head.len() + 1 + data.len();
                        let mut buf = Vec::with_capacity(4 + len);

                        buf.write_u32(len as u32).unwrap();
                        buf.write_u8(ERL_PASS_THROUGH).unwrap();
                        buf.write_u8(ERL_VERSION_MAGIC).unwrap();
                        buf.write_slice(head.as_slice()).unwrap();
                        buf.write_u8(ERL_VERSION_MAGIC).unwrap();
                        buf.write_slice(data.as_slice()).unwrap();

                        try!(stream.write_slice(buf.as_slice()));
                    }
                },

                _ =>
                    unimplemented!(),
            }
        }
    }
}
