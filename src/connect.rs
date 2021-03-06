use std::io;
use std::net;

use consts::*;
use error;
use md5;
use net::{ReadExt, WriteExt};

pub fn accept(stream: &mut net::TcpStream, nodename: &String, cookie: &String) -> Result<(), error::Error>{

    let version = try!(recv_version(stream));

    try!(send_status(stream, "ok"));

    let our_challenge = md5::digest_u32(&["challenge".as_bytes()]); // TODO

    try!(send_challenge(stream, nodename, version, our_challenge));

    let mut expected_digest = [0; 16];
    md5::digest(&[cookie.as_bytes(), our_challenge.to_string().as_bytes()], &mut expected_digest);

    let her_challenge = try!(recv_challenge_reply(stream, &expected_digest));

    let mut our_digest = [0; 16];
    md5::digest(&[cookie.as_bytes(), her_challenge.to_string().as_bytes()], &mut our_digest);

    send_challenge_ack(stream, &our_digest)
}

fn recv_challenge_reply(r: &mut io::Read, expected: &[u8]) -> Result<u32, error::Error> {

    if try!(r.read_u16()) != 21 {
        return Err(from_raw_os_error!(EIO));
    }

    let mut cursor = io::Cursor::new(try!(r.read_vec(21)));

    if cursor.read_u8().unwrap() != 0x72 { // 114: 'r'
        return Err(from_raw_os_error!(EIO));
    }

    let challenge = cursor.read_u32().unwrap();

    if cursor.read_vec(16).unwrap() != expected {
        return Err(from_raw_os_error!(EIO));
    }

    Ok(challenge)
}

fn recv_version(r: &mut io::Read) -> Result<u16, error::Error> {

    let size = try!(r.read_u16()) as usize;

    if size > 7 + MAXNODELEN {
        return Err(from_raw_os_error!(EIO));
    }

    let mut cursor = io::Cursor::new(try!(r.read_vec(size)));

    if cursor.read_u8().unwrap() != 0x6e { // 110: 'n'
        return Err(from_raw_os_error!(EIO));
    }

    let version = cursor.read_u16().unwrap();

    let flags = cursor.read_u32().unwrap();

    for e in vec![
        DFLAG_EXTENDED_REFERENCES,
        DFLAG_EXTENDED_PIDS_PORTS,
    ] {
        if flags & e == 0 {
            return Err(from_raw_os_error!(EIO));
        }
    }

    // cursor.read_vec(size - 7).unwrap(); // peer-node

    Ok(version)
}

fn send_challenge(w: &mut io::Write, name: &str, version: u16, challenge: u32) -> Result<(), error::Error> {
    let len = name.len();
    let mut buf = Vec::with_capacity(11 + len);
    buf.write_u16((11 + len) as u16).unwrap();
    buf.write_u8 (0x6e).unwrap(); // 110: 'n'
    buf.write_u16(version).unwrap();
    buf.write_u32(0
                  | DFLAG_EXTENDED_REFERENCES
                  | DFLAG_DIST_MONITOR
                  | DFLAG_EXTENDED_PIDS_PORTS
                  | DFLAG_FUN_TAGS
                  | DFLAG_NEW_FUN_TAGS
                  | DFLAG_NEW_FLOATS
                  | DFLAG_SMALL_ATOM_TAGS
                  | DFLAG_UTF8_ATOMS
                  | DFLAG_MAP_TAG
                  | DFLAG_BIG_CREATION).unwrap();
    buf.write_u32(challenge).unwrap();
    buf.write_slice(name.as_bytes()).unwrap();
    w.write_slice(buf.as_slice())
}

fn send_challenge_ack(w: &mut io::Write, digest: &[u8]) -> Result<(), error::Error> {
    let mut buf = Vec::with_capacity(17);
    buf.write_u16(17).unwrap();
    buf.write_u8(0x61).unwrap(); //  97: 'a'
    buf.write_slice(digest).unwrap();
    w.write_slice(buf.as_slice())
}

fn send_status(w: &mut io::Write, status: &str) -> Result<(), error::Error> {
    let len = status.len();
    let mut buf = Vec::with_capacity(1 + len);
    buf.write_u16((1 + len) as u16).unwrap();
    buf.write_u8(0x73).unwrap(); // 115: 's'
    buf.write_slice(status.as_bytes()).unwrap();
    w.write_slice(buf.as_slice())
}
