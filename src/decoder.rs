use std::{i16, i32, i64, i8, u16, u32, u64, u8};
use std::io;
use std::mem;

use rustc_serialize;

use consts::*;
use error;
use net::ReadExt;
use term;

pub fn decode<T: rustc_serialize::Decodable>(r: &mut io::Read) -> Result<T, error::Error> {
    T::decode(&mut Decoder { r: r })
}

enum Num {
    U8(u8),
    I27(i32),
    U64(u64, u8),
}

struct Decoder<'a> {
    r: &'a mut io::Read,
}

impl<'a> rustc_serialize::Decoder for Decoder<'a> {

    type Error = error::Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        unimplemented!()
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as u64),
            Num::I27(i)    if range!(i, u32::MIN, i32::MAX, i32) => Ok(i as u64), // != i27
            Num::U64(u, 0)                                       => Ok(u),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as u32),
            Num::I27(i)    if range!(i, u32::MIN, i32::MAX, i32) => Ok(i as u32), // != i27
            Num::U64(u, 0) if range!(u, u32::MIN, u32::MAX, u64) => Ok(u as u32),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as u16),
            Num::I27(i)    if range!(i, u16::MIN, u16::MAX, i32) => Ok(i as u16),
            Num::U64(u, 0) if range!(u, u16::MIN, u16::MAX, u64) => Ok(u as u16),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u) => Ok(u),
            _          => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        unimplemented!()
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as i64),
            Num::I27(i)                                          => Ok(i as i64),
            Num::U64(u, 0) if range!(u, u64::MIN, i64::MAX, u64) => Ok(u as i64),
            Num::U64(u, 1) if range!(u, u64::MIN, i64::MAX, u64) => Ok(u as i64 * -1),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as i32),
            Num::I27(i)                                          => Ok(i),
            Num::U64(u, 0) if range!(u, u32::MIN, i32::MAX, u64) => Ok(u as i32),
            Num::U64(u, 1) if range!(u, u32::MIN, i32::MAX, u64) => Ok(u as i32 * -1),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)                                           => Ok(u as i16),
            Num::I27(i)    if range!(i, i16::MIN, i16::MAX, i32) => Ok(i as i16),
            Num::U64(u, 0) if range!(u, u16::MIN, i16::MAX, u64) => Ok(u as i16),
            _                                                    => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        match try!(self.read_num()) {
            Num::U8(u)     if range!(u, u8::MIN, i8::MAX,  u8) => Ok(u as i8),
            Num::I27(i)    if range!(i, i8::MIN, i8::MAX, i32) => Ok(i as i8),
            Num::U64(u, 0) if range!(u, u8::MIN, i8::MAX, u64) => Ok(u as i8),
            _                                                  => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        match try!(decode(self.r)) {
            term::Atom::Latin1(ref s) => Ok(s == "true"),
            _                         => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        match try!(self.r.read_u8()) {
            NEW_FLOAT_EXT => self.r.read_f64(),
            _             => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        unimplemented!()
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        unimplemented!()
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        match try!(self.r.read_u8()) {
            ERL_NIL_EXT    => Ok("".to_string()),
            ERL_STRING_EXT => self.read_str(),
            ERL_LIST_EXT   => unimplemented!(), // TODO
            _              => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error> {
        unimplemented!()
    }

    fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unimplemented!()
    }

    fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error> {
        unimplemented!()
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        unimplemented!()
    }

    fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        match _s_name {
            "Trace" => self.read_tuple_struct(_s_name, _len, f), // TODO
            _       => f(self),
        }
    }

    fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _len: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        match try!(self.r.read_u8()) {
            ERL_SMALL_TUPLE_EXT => self.r.read_u8().and_then(|_| f(self)),
            ERL_LARGE_TUPLE_EXT => self.r.read_u32().and_then(|_| f(self)),
            _                   => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _s_name: &str, len: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, a_idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        self.read_tuple_arg(a_idx, f)
    }

    fn read_option<T, F>(&mut self, _f: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Self::Error> {
        unimplemented!()
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error> {
        match try!(self.r.read_u8()) {
            ERL_NIL_EXT  => f(self, 0),
            ERL_LIST_EXT => self.r.read_u32().and_then(|u| f(self, u as usize)).and_then(|t| {
                if try!(self.r.read_u8()) == ERL_NIL_EXT {
                    Ok(t)
                } else {
                    Err(from_raw_os_error!(EIO))
                }
            }),
            _  => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error> {
        match try!(self.r.read_u8()) {
            ERL_MAP_EXT => self.r.read_u32().and_then(|u| f(self, u as usize)),
            _           => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error> {
        f(self)
    }

    fn error(&mut self, _err: &str) -> Self::Error {
        unimplemented!()
    }
}

impl<'a> Decoder<'a> {

    fn read_num(&mut self) -> Result<Num, error::Error> {
        match try!(self.r.read_u8()) {
            ERL_SMALL_INTEGER_EXT => self.read_u8(),
            ERL_INTEGER_EXT       => self.read_i27(),
            ERL_SMALL_BIG_EXT     => self.read_small_big(),
            _                     => Err(from_raw_os_error!(EIO)),
        }
    }

    fn read_u8(&mut self) -> Result<Num, error::Error> {
        self.r.read_u8().and_then(|u| Ok(Num::U8(u)))
    }

    fn read_i27(&mut self) -> Result<Num, error::Error> {
        self.r.read_i32().and_then(|i| Ok(Num::I27(i)))
    }

    fn read_small_big(&mut self) -> Result<Num, error::Error> {
        self.r.read_u8().and_then(|a| match a {
            a if a > 8 => Err(from_raw_os_error!(ERANGE)),
            a          => {
                let s = try!(self.r.read_u8());
                self.r.read_vec(a as usize)
                    .and_then(|v| {
                        Ok(v.iter().enumerate().fold(0u64, |a,(i,e)| a | (*e as u64) << (i as u32 * 8)))
                    })
                    .and_then(|u| Ok(Num::U64(u, s)))
            },
        })
    }

    fn read_str(&mut self) -> Result<String, error::Error> {
        self.r.read_u16().and_then(|u| self.r.read_string(u as usize))
    }
}

trait DecoderExt<'a> {
    fn r(&mut self) -> &'a mut io::Read;
}

impl<'a, T: rustc_serialize::Decoder> DecoderExt<'a> for T {

    fn r(&mut self) -> &'a mut io::Read {
        let decoder: &'a mut Decoder<'a> = unsafe { mem::transmute(self) };
        decoder.r
    }
}

// TODO: E::Error = <T as rustc_serialize:Encoder>::Error, -> error.Error

fn read_atom(r: &mut io::Read) -> Result<term::Atom, error::Error> {

    let t = try!(r.read_u8());

    let n = match t {
        ERL_ATOM_EXT            => try!(r.read_u16()) as usize,
        ERL_ATOM_UTF8_EXT       => try!(r.read_u16()) as usize,
        ERL_SMALL_ATOM_UTF8_EXT => try!(r.read_u8()) as usize,
        _                       => return Err(from_raw_os_error!(EIO)),
    };

    match String::from_utf8(try!(r.read_vec(n))) {
        Ok(s) => match t {
            ERL_ATOM_EXT            if s.len() < MAXATOMLEN      => Ok(term::Atom::Latin1(s)),
            ERL_ATOM_UTF8_EXT       if s.len() < MAXATOMLEN_UTF8 => Ok(term::Atom::UTF8(s)),
            ERL_SMALL_ATOM_UTF8_EXT                              => Ok(term::Atom::UTF8Small(s)),
            _                                                    => unreachable!(),
        },
        Err(e) => Err(From::from(e)),
    }
}

impl rustc_serialize::Decodable for term::Atom {

    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("Atom", |d| read_atom(d.r()).or_else(|_| unimplemented!()))
    }
}

fn read_msg(r: &mut io::Read) -> Result<term::Msg, error::Error> {

    if try!(r.read_u8()) != ERL_SMALL_TUPLE_EXT {
        return Err(from_raw_os_error!(EIO));
    }

    let a = try!(r.read_u8());

    match try!(decode(r)) {
        ERL_SEND if a == 3 =>
            Ok(term::Msg::Send {
                cookie: try!(decode(r)),
                to: try!(decode(r)),
            }),
        ERL_SEND_TT if a == 4 =>
            Ok(term::Msg::SendTT {
                cookie: try!(decode(r)),
                to: try!(decode(r)),
                token: try!(decode(r)),
            }),
        ERL_REG_SEND if a == 4 =>
            Ok(term::Msg::RegSend {
                from: try!(decode(r)),
                cookie: try!(decode(r)),
                toname: try!(decode(r)),
            }),
        ERL_REG_SEND_TT if a == 5 =>
            Ok(term::Msg::RegSendTT {
                from: try!(decode(r)),
                cookie: try!(decode(r)),
                toname: try!(decode(r)),
                token: try!(decode(r)),
            }),
        ERL_EXIT if a == 4 =>
            Ok(term::Msg::Exit {
                from: try!(decode(r)),
                to: try!(decode(r)),
                reason: try!(decode(r)),
            }),
        ERL_EXIT_TT if a == 5 =>
            Ok(term::Msg::ExitTT {
                from: try!(decode(r)),
                to: try!(decode(r)),
                token: try!(decode(r)),
                reason: try!(decode(r)),
            }),
        _ => Err(from_raw_os_error!(EIO))
    }
}

impl rustc_serialize::Decodable for term::Msg {

    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("Msg", |d| read_msg(d.r()).or_else(|_| unimplemented!()))
    }
}

fn read_pid(r: &mut io::Read) -> Result<term::Pid, error::Error> {

    let t: bool = match try!(r.read_u8()) {
        ERL_NEW_PID_EXT => true,
        ERL_PID_EXT     => false,
        _               => return Err(from_raw_os_error!(EIO)),
    };

    Ok(term::Pid {
        node: try!(decode(r)),
        num: try!(r.read_u32()) & 0x00007fff,
        serial: try!(r.read_u32()) & 0x00001fff,
        creation: if t { try!(r.read_u32()) } else { try!(r.read_u8()) as u32 & 0x03 },
    })
}

impl rustc_serialize::Decodable for term::Pid {

    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Pid", 4, |d| read_pid(d.r()).or_else(|_| unimplemented!()))
    }
}

fn read_port(r: &mut io::Read) -> Result<term::Port, error::Error> {

    let t = match try!(r.read_u8()) {
        ERL_NEW_PORT_EXT => true,
        ERL_PORT_EXT     => false,
        _                => return Err(from_raw_os_error!(EIO)),
    };

    Ok(term::Port {
        node: try!(decode(r)),
        id: try!(r.read_u32()) & 0x0fffffff,
        creation: if t { try!(r.read_u32()) } else { try!(r.read_u8()) as u32 & 0x03 },
    })
}

impl rustc_serialize::Decodable for term::Port {

    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Port", 3, |d| read_port(d.r()).or_else(|_| unimplemented!()))
    }
}

fn read_ref(r: &mut io::Read) -> Result<term::Ref, error::Error> {

    let t = match try!(r.read_u8()) {
        ERL_NEWER_REFERENCE_EXT => true,
        ERL_NEW_REFERENCE_EXT   => false,
        _                       => return Err(from_raw_os_error!(EIO)),
    };

    let len = try!(r.read_i16()) & 0x0003;

    let node = try!(decode(r));

    let creation = if t { try!(r.read_u32()) } else { try!(r.read_u8()) as u32 & 0x03 };

    let mut n = [0u32; 3];
    for i in 0 .. len as usize {
        n[i] = try!(r.read_u32());
    }

    Ok(term::Ref { len: len, node: node, creation: creation, n: n })
}

impl rustc_serialize::Decodable for term::Ref {

    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Ref", 4, |d| read_ref(d.r()).or_else(|_| unimplemented!()))
    }
}

#[cfg(test)]
mod tests {

    use std::collections;

    use term;

    macro_rules! test {
        ($i: expr) => (super::decode(&mut $i.as_slice()));
        ($i: expr, $t: ty) => (super::decode::<$t>(&mut $i.as_slice()));
    }

    #[test]
    fn decode_u64() {
        for (expected, input) in vec![
            (                   0 as u64, vec![0x61, 0x00]),
            (                   1 as u64, vec![0x61, 0x01]),
            (                 255 as u64, vec![0x61, 0xff]),
            (                 256 as u64, vec![0x62, 0x00,0x00,0x01,0x00]),
            (           134217727 as u64, vec![0x62, 0x07,0xff,0xff,0xff]),
            (           134217728 as u64, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            (18446744073709551615 as u64, vec![0x6e, 0x08, 0x00, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("18446744073709551616", vec![0x6e, 0x09, 0x00, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01]),
        ] {
            assert!(test!(input, u64).is_err(), expected);
        }
    }

    #[test]
    fn decode_u32() {
        for (expected, input) in vec![
            (         0 as u32, vec![0x61, 0x00]),
            (         1 as u32, vec![0x61, 0x01]),
            (       255 as u32, vec![0x61, 0xff]),
            (       256 as u32, vec![0x62, 0x00,0x00,0x01,0x00]),
            ( 134217727 as u32, vec![0x62, 0x07,0xff,0xff,0xff]),
            ( 134217728 as u32, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            (4294967295 as u32, vec![0x6e, 0x04, 0x00, 0xff,0xff,0xff,0xff]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("4294967296", vec![0x6e, 0x05, 0x00, 0x00,0x00,0x00,0x00,0x01]),
        ] {
            assert!(test!(input, u32).is_err(), expected);
        }
    }

    #[test]
    fn decode_u16() {
        for (expected, input) in vec![
            (    0 as u16, vec![0x61, 0x00]),
            (    1 as u16, vec![0x61, 0x01]),
            (  255 as u16, vec![0x61, 0xff]),
            (  256 as u16, vec![0x62, 0x00,0x00,0x01,0x00]),
            (65535 as u16, vec![0x62, 0x00,0x00,0xff,0xff]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("65536", vec![0x62, 0x00,0x01,0x00,0x00]),
        ] {
            assert!(test!(input, u16).is_err(), expected);
        }
    }

    #[test]
    fn decode_u8() {
        for (expected, input) in vec![
            (  0 as u8, vec![0x61, 0x00]),
            (  1 as u8, vec![0x61, 0x01]),
            (255 as u8, vec![0x61, 0xff]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("256", vec![0x62, 0x00,0x00,0x01,0x00]),
        ] {
            assert!(test!(input, u8).is_err(), expected);
        }
    }

    #[test]
    fn decode_i64() {
        for (expected, input) in vec![
            (-9223372036854775807 as i64, vec![0x6e, 0x08, 0x01, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f]),
            (          -134217729 as i64, vec![0x6e, 0x04, 0x01, 0x01,0x00,0x00,0x08]),
            (          -134217728 as i64, vec![0x62, 0xf8,0x00,0x00,0x00]),
            (                  -1 as i64, vec![0x62, 0xff,0xff,0xff,0xff]),
            (                   0 as i64, vec![0x61, 0x00]),
            (                   1 as i64, vec![0x61, 0x01]),
            (                 255 as i64, vec![0x61, 0xff]),
            (                 256 as i64, vec![0x62, 0x00,0x00,0x01,0x00]),
            (           134217727 as i64, vec![0x62, 0x07,0xff,0xff,0xff]),
            (           134217728 as i64, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            ( 9223372036854775807 as i64, vec![0x6e, 0x08, 0x00, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("-9223372036854775809", vec![0x6e, 0x09, 0x01, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xf7,0xff]),
            ("-9223372036854775808", vec![0x6e, 0x08, 0x01, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x80]),
            ( "9223372036854775808", vec![0x6e, 0x09, 0x00, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x80,0x00]),
        ] {
            assert!(test!(input, i64).is_err(), expected);
        }
    }

    #[test]
    fn decode_i32() {
        for (expected, input) in vec![
            (-2147483647 as i32, vec![0x6e, 0x04, 0x01, 0xff,0xff,0xff,0x7f]),
            ( -134217729 as i32, vec![0x6e, 0x04, 0x01, 0x01,0x00,0x00,0x08]),
            ( -134217728 as i32, vec![0x62, 0xf8,0x00,0x00,0x00]),
            (         -1 as i32, vec![0x62, 0xff,0xff,0xff,0xff]),
            (          0 as i32, vec![0x61, 0x00]),
            (          1 as i32, vec![0x61, 0x01]),
            (        255 as i32, vec![0x61, 0xff]),
            (        256 as i32, vec![0x62, 0x00,0x00,0x01,0x00]),
            (  134217727 as i32, vec![0x62, 0x07,0xff,0xff,0xff]),
            (  134217728 as i32, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            ( 2147483647 as i32, vec![0x6e, 0x04, 0x00, 0xff,0xff,0xff,0x7f]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("-2147483649", vec![0x6e, 0x05, 0x01, 0xff,0xff,0xff,0x7f,0xff]),
            ("-2147483648", vec![0x6e, 0x05, 0x01, 0x00,0x00,0x00,0x80,0x00]),
            ( "2147483648", vec![0x6e, 0x05, 0x00, 0x00,0x00,0x00,0x80,0x00]),
        ] {
            assert!(test!(input, i32).is_err(), expected);
        }
    }

    #[test]
    fn decode_i16() {
        for (expected, input) in vec![
            (-32768 as i16, vec![0x62, 0xff,0xff,0x80,0x00]),
            (    -1 as i16, vec![0x62, 0xff,0xff,0xff,0xff]),
            (     0 as i16, vec![0x61, 0x00]),
            (     1 as i16, vec![0x61, 0x01]),
            (   255 as i16, vec![0x61, 0xff]),
            (   256 as i16, vec![0x62, 0x00,0x00,0x01,0x00]),
            ( 32767 as i16, vec![0x62, 0x00,0x00,0x7f,0xff]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("-32769", vec![0x62, 0xff,0xff,0x7f,0xff]),
            ( "32768", vec![0x62, 0x00,0x00,0x80,0x00]),
        ] {
            assert!(test!(input, i16).is_err(), expected);
        }
    }

    #[test]
    fn decode_i8() {
        for (expected, input) in vec![
            (-128 as i8, vec![0x62, 0xff,0xff,0xff,0x80]),
            (  -1 as i8, vec![0x62, 0xff,0xff,0xff,0xff]),
            (   0 as i8, vec![0x61, 0x00]),
            (   1 as i8, vec![0x61, 0x01]),
            ( 127 as i8, vec![0x61, 0x7f]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
        for (expected, input) in vec![
            ("-129", vec![0x62, 0xff,0xff,0xff,0x7f]),
            ( "128", vec![0x61, 0x80]),
        ] {
            assert!(test!(input, i8).is_err(), expected);
        }
    }

    #[test]
    fn encode_bool() {
        for (expected, input) in vec![
            (true,  vec![0x64, 0x00,0x04, 0x74,0x72,0x75,0x65]),
            (false, vec![0x64, 0x00,0x05, 0x66,0x61,0x6c,0x73,0x65]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_f64() {
        for (expected, input) in vec![
            (-1.0 as f64, vec![0x46, 0xbf,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
            (-0.0 as f64, vec![0x46, 0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
            ( 0.0 as f64, vec![0x46, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
            ( 1.0 as f64, vec![0x46, 0x3f,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_str() {
        for (expected, input) in vec![
            ("",      vec![0x6a]),
            ("hello", vec![0x6b, 0x00,0x05, 0x68,0x65,0x6c,0x6c,0x6f]),
        ] {
            assert_eq!(expected, test!(input, String).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_tuple() {
        assert_eq!((),        test!(vec![0x6a]).unwrap());
        assert_eq!((1,),      test!(vec![0x68, 0x01, 0x61,0x01]).unwrap());
        assert_eq!((1, true), test!(vec![0x68, 0x02, 0x61,0x01, 0x64,0x00,0x04,0x74,0x72,0x75,0x65]).unwrap());
    }

    #[test]
    fn decode_seq() {
        for (expected, input) in vec![
            (vec![],  vec![0x6a]),
            (vec![1], vec![0x6c, 0x00,0x00,0x00,0x01, 0x61,0x01, 0x6a]),
        ] {
            assert_eq!(expected, test!(input, Vec<u8>).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_map() {

        let mut map = collections::HashMap::new();

        map.clear();
        assert_eq!(map, test!(vec![0x74, 0x00,0x00,0x00,0x00]).unwrap());

        map.clear();
        map.insert(1, 2);
        assert_eq!(map, test!(vec![0x74, 0x00,0x00,0x00,0x01, 0x61,0x01, 0x61,0x02]).unwrap());
    }


    #[test]
    fn decode_atom() {
        for (expected, input) in vec![
            (
                term::Atom::from("true"),
                vec![0x64, 0x00,0x04, 0x74,0x72,0x75,0x65]
            ),
            (
                term::Atom::Latin1("false".to_string()),
                vec![0x64, 0x00,0x05, 0x66,0x61,0x6c,0x73,0x65]
            ),
            (
                term::Atom::UTF8("n1".to_string()),
                vec![0x76, 0x00,0x02, 0x6e,0x31]
            ),
            (
                term::Atom::UTF8Small("n1".to_string()),
                vec![0x77, 0x02, 0x6e,0x31]
            ),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_msg() {
        for (expected, input) in vec![
            (
                term::Msg::Send {
                    cookie: term::Atom::UTF8Small("".to_string()),
                    to: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                },
                vec![0x68, 0x03,
                     0x61, 0x02,
                     0x77, 0x00,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03]
            ),
            (
                term::Msg::SendTT {
                    cookie: term::Atom::UTF8Small("".to_string()),
                    to: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    token: term::Trace (
                        1,                                // 0: flags
                        2,                                // 1: label
                        4,                                // 2: serial
                        term::Pid {                       // 3: from
                            node: term::Atom::from("n1"),
                            num: 1,
                            serial: 2,
                            creation: 4,
                        },
                        8,                                // 4: prev
                    ),
                },
                vec![0x68, 0x04,
                     0x61, 0x0c,
                     0x77, 0x00,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x68, 0x05,0x61,0x01,0x61,0x02,0x61,0x04,0x58,0x64,0x00,0x02,0x6e,0x31,
                     /* */ 0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x04,0x61,0x08]
            ),
            (
                term::Msg::RegSend {
                    from: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    cookie: term::Atom::UTF8Small("".to_string()),
                    toname: term::Atom::UTF8Small("any".to_string()),
                },
                vec![0x68, 0x04,
                     0x61, 0x06,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x77, 0x00,
                     0x77, 0x03,0x61,0x6e,0x79]
            ),
            (
                term::Msg::RegSendTT {
                    from: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    cookie: term::Atom::UTF8Small("".to_string()),
                    toname: term::Atom::UTF8Small("any".to_string()),
                    token: term::Trace (
                        1,                                // 0: flags
                        2,                                // 1: label
                        4,                                // 2: serial
                        term::Pid {                       // 3: from
                            node: term::Atom::from("n1"),
                            num: 1,
                            serial: 2,
                            creation: 4,
                        },
                        8,                                // 4: prev
                    ),
                },
                vec![0x68, 0x05,
                     0x61, 0x10,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x77, 0x00,
                     0x77, 0x03,0x61,0x6e,0x79,
                     0x68, 0x05,0x61,0x01,0x61,0x02,0x61,0x04,0x58,0x64,0x00,0x02,0x6e,0x31,
                     /* */ 0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x04,0x61,0x08]
            ),
            (
                term::Msg::Exit {
                    from: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    to: term::Pid {
                        node: term::Atom::from("r1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    reason: "any".to_string(),
                },
                vec![0x68, 0x04,
                     0x61, 0x03,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x67, 0x64,0x00,0x02,0x72,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x6b, 0x00,0x03,0x61,0x6e,0x79]
            ),
            (
                term::Msg::ExitTT {
                    from: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    to: term::Pid {
                        node: term::Atom::from("r1"),
                        num: 0x00007fff,
                        serial: 0x00001fff,
                        creation: 0x00000003,
                    },
                    token: term::Trace (
                        1,                                // 0: flags
                        2,                                // 1: label
                        4,                                // 2: serial
                        term::Pid {                       // 3: from
                            node: term::Atom::from("n1"),
                            num: 1,
                            serial: 2,
                            creation: 4,
                        },
                        8,                                // 4: prev
                    ),
                    reason: "any".to_string(),
                },
                vec![0x68, 0x05,
                     0x61, 0x0d,
                     0x67, 0x64,0x00,0x02,0x6e,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x67, 0x64,0x00,0x02,0x72,0x31,0x00,0x00,0x7f,0xff,0x00,0x00,0x1f,0xff,0x03,
                     0x68, 0x05,0x61,0x01,0x61,0x02,0x61,0x04,0x58,0x64,0x00,0x02,0x6e,0x31,
                     /* */ 0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x04,0x61,0x08,
                     0x6b, 0x00,0x03,0x61,0x6e,0x79]
            ),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_pid() {
        for (expected, input) in vec![
            (
                term::Pid {
                    node: term::Atom::from("n1"),
                    num: 0x00007fff,
                    serial: 0x00001fff,
                    creation: 0x00000003,
                },
                vec![0x67,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0xff,0xff,0xff,0xff,
                     0xff,0xff,0xff,0xff,
                     0x03]
            ),
            (
                term::Pid {
                    node: term::Atom::from("n1"),
                    num: 0x00007fff,
                    serial: 0x00001fff,
                    creation: 0x00000004,
                },
                vec![0x58,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0xff,0xff,0xff,0xff,
                     0xff,0xff,0xff,0xff,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_port() {
        for (expected, input) in vec![
            (
                term::Port {
                    node: term::Atom::from("n1"),
                    id: 0x0fffffff,
                    creation: 0x00000003,
                },
                vec![0x66,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0xff,0xff,0xff,0xff,
                     0x03]
            ),
            (
                term::Port {
                    node: term::Atom::from("n1"),
                    id: 0x0fffffff,
                    creation: 0x00000004,
                },
                vec![0x59,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0xff,0xff,0xff,0xff,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_ref() {
        for (expected, input) in vec![
            (
                term::Ref {
                    len: 0,
                    node: term::Atom::from("n1"),
                    creation: 3,
                    n: [0,0,0],
                },
                vec![0x72,
                     0x00,0x00,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x03]
            ),
            (
                term::Ref {
                    len: 1,
                    node: term::Atom::from("n1"),
                    creation: 3,
                    n: [10,0,0],
                },
                vec![0x72,
                     0x00,0x01,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x03,
                     0x00,0x00,0x00,0x0a]
            ),
            (
                term::Ref {
                    len: 3,
                    node: term::Atom::from("n1"),
                    creation: 3,
                    n: [10,11,12],
                },
                vec![0x72,
                     0x00,0x03,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x03,
                     0x00,0x00,0x00,0x0a,0x00,0x00,0x00,0x0b,0x00,0x00,0x00,0x0c]
            ),
            (
                term::Ref {
                    len: 0,
                    node: term::Atom::from("n1"),
                    creation: 4,
                    n: [0,0,0],
                },
                vec![0x5a,
                     0x00,0x00,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn decode_trace() {
        for (expected, input) in vec![
            (
                term::Trace (
                    1,                                // 0: flags
                    2,                                // 1: label
                    4,                                // 2: serial
                    term::Pid {                       // 3: from
                        node: term::Atom::from("n1"),
                        num: 1,
                        serial: 2,
                        creation: 4,
                    },
                    8,                                // 4: prev
                ),
                vec![0x68, 0x05,
                     0x61, 0x01,
                     0x61, 0x02,
                     0x61, 0x04,
                     0x58,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x00,0x00,0x00,0x01,
                     0x00,0x00,0x00,0x02,
                     0x00,0x00,0x00,0x04,
                     0x61, 0x08]
            )
        ] {
            assert_eq!(expected, test!(input).unwrap(), "{:?}", input);
        }
    }

    // let n = input.len() as u64;
    // let mut cursor = io::Cursor::new(input);
    // assert_eq!(expected, ni::decode(&mut cursor).unwrap(), "{:?}", input);
    // assert_eq!(n, c.position());
}
