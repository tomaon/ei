use std::{i32, i64, i8, u16, u32, u64, u8, usize};
use std::io;
use std::mem;

use rustc_serialize;

use consts::*;
use error;
use i27;
use net::WriteExt;
use term;

pub fn encode<T: rustc_serialize::Encodable>(w: &mut io::Write, t: &T) -> Result<(), error::Error> {
    t.encode(&mut Encoder { w: w })
}

struct Encoder<'a> {
    w: &'a mut io::Write,
}

impl<'a> rustc_serialize::Encoder for Encoder<'a> {

    type Error = error::Error;

    fn emit_nil(&mut self) -> Result<(), Self::Error> {
        self.w.write_u8(ERL_NIL_EXT)
    }

    fn emit_usize(&mut self, _v: usize) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        match v {
            u if range!(u,  u8::MIN,  u8::MAX, u64) => self.write_u8(u as u8),
            u if range!(u, u32::MIN, i27::MAX, u64) => self.write_i27(u as i32),
            u                                       => self.write_small_big(0, u as u64),
        }
    }

    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        match v {
            u if range!(u,  u8::MIN,  u8::MAX, u32) => self.write_u8(u as u8),
            u if range!(u, u32::MIN, i27::MAX, u32) => self.write_i27(u as i32),
            u                                       => self.write_small_big(0, u as u64),
        }
    }

    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        match v {
            u if range!(u, u8::MIN, u8::MAX, u16) => self.write_u8(u as u8),
            u                                     => self.write_i27(u as i32),
        }
    }

    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        self.write_u8(v)
    }

    fn emit_isize(&mut self, _v: isize) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        match v {
            i if i == i64::MIN                      => Err(from_raw_os_error!(EDOM)),
            i if range!(i,  u8::MIN,  u8::MAX, i64) => self.write_u8(i as u8),
            i if range!(i, i27::MIN, i27::MAX, i64) => self.write_i27(i as i32),
            i if range!(i, i64::MIN, u64::MIN, i64) => self.write_small_big(1, i.abs() as u64),
            i                                       => self.write_small_big(0, i as u64),
        }
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        match v {
            i if i == i32::MIN                      => Err(from_raw_os_error!(EDOM)),
            i if range!(i,  u8::MIN,  u8::MAX, i32) => self.write_u8(i as u8),
            i if range!(i, i27::MIN, i27::MAX, i32) => self.write_i27(i as i32),
            i if range!(i, i32::MIN, u32::MIN, i32) => self.write_small_big(1, i.abs() as u64),
            i                                       => self.write_small_big(0, i as u64),
        }
    }

    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        match v {
            i if range!(i, u8::MIN, u8::MAX, i16) => self.write_u8(i as u8),
            i                                     => self.write_i27(i as i32),
        }
    }

    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        match v {
            i if range!(i, u8::MIN, i8::MAX, i8) => self.write_u8(i as u8),
            i                                    => self.write_i27(i as i32),
        }
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        encode(self.w, &term::Atom::from(if v { "true" } else { "false" }))
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        self.w.write_u8(NEW_FLOAT_EXT).and_then(|()| self.w.write_f64(v))
    }

    fn emit_f32(&mut self, _v: f32) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_char(&mut self, _v: char) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        match v.len() {
            0                          => self.emit_nil(),
            n if n > u16::MAX as usize => unimplemented!(), // TODO
            n                          => self.write_str(v, n),
        }
    }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }

    fn emit_enum_variant<F>(&mut self, _v_name: &str, _v_id: usize, _len: usize, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_enum_variant_arg<F>(&mut self, _a_idx: usize, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_enum_struct_variant<F>(&mut self, _v_name: &str, _v_id: usize, _len: usize, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_enum_struct_variant_field<F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_struct<F>(&mut self, _name: &str, _len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        match _name {
            "Trace" => self.emit_tuple_struct(_name, _len, f), // TODO
            _       => f(self),
        }
    }

    fn emit_struct_field<F>(&mut self, _f_name: &str, _f_idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        if len > u8::MAX as usize {
            try!(self.w.write_u8(ERL_LARGE_TUPLE_EXT).and_then(|()| self.w.write_u32(len as u32)));
        } else {
            try!(self.w.write_u8(ERL_SMALL_TUPLE_EXT).and_then(|()| self.w.write_u8(len as u8)));
        }
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        self.emit_tuple(len, f)
    }

    fn emit_tuple_struct_arg<F>(&mut self, f_idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        self.emit_tuple_arg(f_idx, f)
    }

    fn emit_option<F>(&mut self, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_option_none(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_option_some<F>(&mut self, _f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        if len > usize::MIN {
            try!(self.w.write_u8(ERL_LIST_EXT).and_then(|()| self.w.write_u32(len as u32)));
            try!(f(self));
        }
        self.emit_nil()
    }

    fn emit_seq_elt<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        try!(self.w.write_u8(ERL_MAP_EXT).and_then(|()| self.w.write_u32(len as u32)));
        f(self)
    }

    fn emit_map_elt_key<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error> {
        f(self)
    }
}

impl<'a> Encoder<'a> {

    fn write_u8(&mut self, v: u8) -> Result<(), error::Error> {
        self.w.write_slice(&[ERL_SMALL_INTEGER_EXT, v])
    }

    fn write_i27(&mut self, v: i32) -> Result<(), error::Error> {
        self.w.write_u8(ERL_INTEGER_EXT).and_then(|()| self.w.write_i32(v))
    }

    fn write_small_big(&mut self, s: u8, v: u64) -> Result<(), error::Error> {
        let mut buf: Vec<u8> = Vec::with_capacity(8);
        let mut x = v;
        while x != 0 {
            buf.push((x & 0xff) as u8);
            x >>= 8;
        }
        buf.shrink_to_fit();
        self.w.write_slice(&[ERL_SMALL_BIG_EXT, buf.len() as u8, s])
            .and_then(|()| self.w.write_slice(buf.as_slice()))
    }

    fn write_str(&mut self, v: &str, len: usize) -> Result<(), error::Error> {
        self.w.write_u8(ERL_STRING_EXT)
            .and_then(|()| self.w.write_u16(len as u16))
            .and_then(|()| self.w.write_slice(v.as_bytes()))
    }
}

trait EncoderExt<'a> {
    fn w(&mut self) -> &'a mut io::Write;
}

impl<'a, T: rustc_serialize::Encoder> EncoderExt<'a> for T {

    fn w(&mut self) -> &'a mut io::Write {
        let encoder: &'a mut Encoder<'a> = unsafe { mem::transmute(self) };
        encoder.w
    }
}

// TODO: S::Error = <S as rustc_serialize:Encoder>::Error, -> error.Error

fn write_atom(w: &mut io::Write, v: &term::Atom) -> Result<(), error::Error> {
    match *v {
        term::Atom::Latin1(ref s) if s.len() < MAXATOMLEN => {
            try!(w.write_u8(ERL_ATOM_EXT));
            try!(w.write_u16(s.len() as u16));
            w.write_slice(s.as_bytes())
        },
        term::Atom::UTF8(ref s) if s.len() < MAXATOMLEN_UTF8 => {
            try!(w.write_u8(ERL_ATOM_UTF8_EXT));
            try!(w.write_u16(s.len() as u16));
            w.write_slice(s.as_bytes())
        },
        term::Atom::UTF8Small(ref s) if s.len() < u8::MAX as usize => {
            try!(w.write_u8(ERL_SMALL_ATOM_UTF8_EXT));
            try!(w.write_u8(s.len() as u8));
            w.write_slice(s.as_bytes())
        },
        _ => Err(from_raw_os_error!(ERANGE)),
    }
}

impl rustc_serialize::Encodable for term::Atom {

    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Atom", |s| write_atom(s.w(), &self).or_else(|_| unimplemented!()))
    }
}

fn write_msg(w: &mut io::Write, v: &term::Msg) -> Result<(), error::Error> {

    try!(w.write_u8(ERL_SMALL_TUPLE_EXT));

    match *v {
        term::Msg::Send { ref cookie, ref to } => {
            try!(w.write_u8(3));
            try!(encode(w, &ERL_SEND));
            try!(encode(w, cookie));
            encode(w, to)
        },
        term::Msg::SendTT { ref cookie, ref to, ref token } => {
            try!(w.write_u8(4));
            try!(encode(w, &ERL_SEND_TT));
            try!(encode(w, cookie));
            try!(encode(w, to));
            encode(w, token)
        },
        term::Msg::RegSend { ref from, ref cookie, ref toname } => {
            try!(w.write_u8(4));
            try!(encode(w, &ERL_REG_SEND));
            try!(encode(w, from));
            try!(encode(w, cookie));
            encode(w, toname)
        },
        term::Msg::RegSendTT { ref from, ref cookie, ref toname, ref token } => {
            try!(w.write_u8(5));
            try!(encode(w, &ERL_REG_SEND_TT));
            try!(encode(w, from));
            try!(encode(w, cookie));
            try!(encode(w, toname));
            encode(w, token)
        },
        term::Msg::Exit { ref from, ref to, ref reason } => {
            try!(w.write_u8(4));
            try!(encode(w, &ERL_EXIT));
            try!(encode(w, from));
            try!(encode(w, to));
            encode(w, reason)
        },
        term::Msg::ExitTT { ref from, ref to, ref token, ref reason } => {
            try!(w.write_u8(5));
            try!(encode(w, &ERL_EXIT_TT));
            try!(encode(w, from));
            try!(encode(w, to));
            try!(encode(w, token));
            encode(w, reason)
        },
    }
}

impl rustc_serialize::Encodable for term::Msg {

    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Msg", |s| write_msg(s.w(), &self).or_else(|_| unimplemented!()))
    }
}

fn write_pid(w: &mut io::Write, v: &term::Pid) -> Result<(), error::Error> {

    if v.creation > 3 {
        try!(w.write_u8(ERL_NEW_PID_EXT));
    } else {
        try!(w.write_u8(ERL_PID_EXT));
    }

    try!(encode(w, &v.node));

    try!(w.write_u32(v.num & 0x00007fff));

    try!(w.write_u32(v.serial & 0x00001fff));

    if v.creation > 3 {
        w.write_u32(v.creation)
    } else {
        w.write_u8(v.creation as u8)
    }
}

impl rustc_serialize::Encodable for term::Pid {

    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Pid", 4, |s| write_pid(s.w(), &self).or_else(|_| unimplemented!()))
    }
}

fn write_port(w: &mut io::Write, v: &term::Port) -> Result<(), error::Error> {

    if v.creation > 3 {
        try!(w.write_u8(ERL_NEW_PORT_EXT));
    } else {
        try!(w.write_u8(ERL_PORT_EXT));
    }

    try!(encode(w, &v.node));

    try!(w.write_u32(v.id & 0x0fffffff));

    if v.creation > 3 {
        w.write_u32(v.creation)
    } else {
        w.write_u8(v.creation as u8)
    }
}

impl rustc_serialize::Encodable for term::Port {

    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Port", 3, |s| write_port(s.w(), &self).or_else(|_| unimplemented!()))
    }
}

fn write_ref(w: &mut io::Write, v: &term::Ref) -> Result<(), error::Error> {

    if v.creation > 3 {
        try!(w.write_u8(ERL_NEWER_REFERENCE_EXT));
    } else {
        try!(w.write_u8(ERL_NEW_REFERENCE_EXT));
    }

    try!(w.write_i16(v.len));

    try!(encode(w, &v.node));

    if v.creation > 3 {
        try!(w.write_u32(v.creation));
    } else {
        try!(w.write_u8(v.creation as u8));
    }

    for i in 0 .. v.len {
        try!(w.write_u32(v.n[i as usize]));
    }

    Ok(())
}

impl rustc_serialize::Encodable for term::Ref {

    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Ref", 4, |s| write_ref(s.w(), &self).or_else(|_| unimplemented!()))
    }
}

#[cfg(test)]
mod tests {

    use std::collections;

    use term;

    macro_rules! test {
        ($o: expr, $i: expr) => (super::encode(&mut $o, &$i).and(Ok($o.as_slice())));
    }

    #[test]
    fn encode_u64() {
        for (input, expected) in vec![
            (                   0 as u64, vec![0x61, 0x00]),
            (                   1 as u64, vec![0x61, 0x01]),
            (                 255 as u64, vec![0x61, 0xff]),
            (                 256 as u64, vec![0x62, 0x00,0x00,0x01,0x00]),
            (           134217727 as u64, vec![0x62, 0x07,0xff,0xff,0xff]),
            (           134217728 as u64, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            (18446744073709551615 as u64, vec![0x6e, 0x08, 0x00, 0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_u32() {
        for (input, expected) in vec![
            (         0 as u32, vec![0x61, 0x00]),
            (         1 as u32, vec![0x61, 0x01]),
            (       255 as u32, vec![0x61, 0xff]),
            (       256 as u32, vec![0x62, 0x00,0x00,0x01,0x00]),
            ( 134217727 as u32, vec![0x62, 0x07,0xff,0xff,0xff]),
            ( 134217728 as u32, vec![0x6e, 0x04, 0x00, 0x00,0x00,0x00,0x08]),
            (4294967295 as u32, vec![0x6e, 0x04, 0x00, 0xff,0xff,0xff,0xff]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_u16() {
        for (input, expected) in vec![
            (    0 as u16, vec![0x61, 0x00]),
            (    1 as u16, vec![0x61, 0x01]),
            (  255 as u16, vec![0x61, 0xff]),
            (  256 as u16, vec![0x62, 0x00,0x00,0x01,0x00]),
            (65535 as u16, vec![0x62, 0x00,0x00,0xff,0xff]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_u8() {
        for (input, expected) in vec![
            (  0 as u8, vec![0x61, 0x00]),
            (  1 as u8, vec![0x61, 0x01]),
            (255 as u8, vec![0x61, 0xff]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_i64() {
        for (input, expected) in vec![
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
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
        for (input, expected) in vec![
            (-9223372036854775808 as i64, vec![0x6e, 0x08, 0x01, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x80]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert!(test!(&mut output, input).is_err(), expected);
        }
    }

    #[test]
    fn encode_i32() {
        for (input, expected) in vec![
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
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
        for (input, expected) in vec![
            (-2147483648 as i32, vec![0x6e, 0x04, 0x01, 0x00,0x00,0x00,0x80]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert!(test!(&mut output, input).is_err(), expected);
        }
    }

    #[test]
    fn encode_i16() {
        for (input, expected) in vec![
            (-32768 as i16, vec![0x62, 0xff,0xff,0x80,0x00]),
            (    -1 as i16, vec![0x62, 0xff,0xff,0xff,0xff]),
            (     0 as i16, vec![0x61, 0x00]),
            (     1 as i16, vec![0x61, 0x01]),
            (   255 as i16, vec![0x61, 0xff]),
            (   256 as i16, vec![0x62, 0x00,0x00,0x01,0x00]),
            ( 32767 as i16, vec![0x62, 0x00,0x00,0x7f,0xff]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_i8() {
        for (input, expected) in vec![
            (-128 as i8, vec![0x62, 0xff,0xff,0xff,0x80]),
            (  -1 as i8, vec![0x62, 0xff,0xff,0xff,0xff]),
            (   0 as i8, vec![0x61, 0x00]),
            (   1 as i8, vec![0x61, 0x01]),
            ( 127 as i8, vec![0x61, 0x7f]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_bool() {
        for (input, expected) in vec![
            (true,  vec![0x64, 0x00,0x04, 0x74,0x72,0x75,0x65]),
            (false, vec![0x64, 0x00,0x05, 0x66,0x61,0x6c,0x73,0x65]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_f64() {
        for (input, expected) in vec![
            (-1.0 as f64, vec![0x46, 0xbf,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
            (-0.0 as f64, vec![0x46, 0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
            ( 0.0 as f64, vec![0x46, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
            ( 1.0 as f64, vec![0x46, 0x3f,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_str() {
        for (input, expected) in vec![
            ("",      vec![0x6a]),
            ("hello", vec![0x6b, 0x00,0x05, 0x68,0x65,0x6c,0x6c,0x6f]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_tuple() {

        let mut output: Vec<u8> = Vec::new();

        output.clear();
        assert_eq!([0x6a], test!(&mut output, ()).unwrap());

        output.clear();
        assert_eq!([0x68, 0x01, 0x61,0x01], test!(&mut output, (1,)).unwrap());

        output.clear();
        assert_eq!([0x68, 0x02, 0x61,0x01, 0x64,0x00,0x04,0x74,0x72,0x75,0x65], test!(&mut output, (1,true)).unwrap());
    }

    #[test]
    fn encode_seq() {
        for (input, expected) in vec![
            (vec![],  vec![0x6a]),
            (vec![1], vec![0x6c, 0x00,0x00,0x00,0x01, 0x61,0x01, 0x6a]),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_map() {

        let mut output: Vec<u8> = Vec::new();

        let mut map = collections::HashMap::new();

        output.clear();
        map.clear();
        assert_eq!([0x74, 0x00,0x00,0x00,0x00], test!(&mut output, &map).unwrap());

        output.clear();
        map.clear();
        map.insert(1, 2);
        assert_eq!([0x74, 0x00,0x00,0x00,0x01, 0x61,0x01, 0x61,0x02], test!(&mut output, &map).unwrap());
    }


    #[test]
    fn encode_atom() {
        for (input, expected) in vec![
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
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_msg() {
        for (input, expected) in vec![
            (
                term::Msg::Send {
                    cookie: term::Atom::UTF8Small("".to_string()),
                    to: term::Pid {
                        node: term::Atom::from("n1"),
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
                        num: 0xffffffff,
                        serial: 0xffffffff,
                        creation: 0x00000003,
                    },
                    to: term::Pid {
                        node: term::Atom::from("r1"),
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
                        num: 0xffffffff,
                        serial: 0xffffffff,
                        creation: 0x00000003,
                    },
                    to: term::Pid {
                        node: term::Atom::from("r1"),
                        num: 0xffffffff,
                        serial: 0xffffffff,
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
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_pid() {
        for (input, expected) in vec![
            (
                term::Pid {
                    node: term::Atom::from("n1"),
                    num: 0xffffffff,
                    serial: 0xffffffff,
                    creation: 0x00000003,
                },
                vec![0x67,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x00,0x00,0x7f,0xff,
                     0x00,0x00,0x1f,0xff,
                     0x03]
            ),
            (
                term::Pid {
                    node: term::Atom::from("n1"),
                    num: 0xffffffff,
                    serial: 0xffffffff,
                    creation: 0x00000004,
                },
                vec![0x58,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x00,0x00,0x7f,0xff,
                     0x00,0x00,0x1f,0xff,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_port() {
        for (input, expected) in vec![
            (
                term::Port {
                    node: term::Atom::from("n1"),
                    id: 0xffffffff,
                    creation: 0x00000003,
                },
                vec![0x66,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x0f,0xff,0xff,0xff,
                     0x03]
            ),
            (
                term::Port {
                    node: term::Atom::from("n1"),
                    id: 0xffffffff,
                    creation: 0x00000004,
                },
                vec![0x59,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x0f,0xff,0xff,0xff,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_ref() {
        for (input, expected) in vec![
            (
                term::Ref {
                    len: 0,
                    node: term::Atom::from("n1"),
                    creation: 3,
                    n: [10,11,12],
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
                    n: [10,11,12],
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
                    n: [10,11,12],
                },
                vec![0x5a,
                     0x00,0x00,
                     0x64, 0x00,0x02, 0x6e,0x31,
                     0x00,0x00,0x00,0x04]
            ),
        ] {
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn encode_trace() {
        for (input, expected) in vec![
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
            let mut output: Vec<u8> = Vec::new();
            assert_eq!(expected, test!(&mut output, input).unwrap(), "{:?}", input);
        }
    }
}
