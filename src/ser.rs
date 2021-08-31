use std::io;

use serde::{ser, serde_if_integer128};

use crate::consts::*;
use crate::error::Error;
use crate::i27;
use crate::io::Writer;

pub struct Serializer<W> {
    writer: Writer<W>,
    etype: Vec<[u8; 2]>,
    ref_n: Option<usize>,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    pub fn new(w: W) -> Self {
        Serializer {
            writer: Writer::new(w),
            etype: Vec::with_capacity(16),
            ref_n: None,
        }
    }

    pub fn write_i27(&mut self, v: i32) -> Result<(), Error> {
        self.writer.write_u8(ERL_INTEGER_EXT)?;
        self.writer.write_i32(v)
    }

    pub fn write_small_big(&mut self, s: u8, v: u64) -> Result<(), Error> {
        let mut vec = Vec::with_capacity(8);
        let mut u = v;
        while u != 0 {
            vec.push((u & 0xff) as u8);
            u >>= 8;
        }
        self.writer
            .write_all(&[ERL_SMALL_BIG_EXT, vec.len() as u8, s])?;
        self.writer.write_all(&vec)
    }

    pub fn write_tuple(&mut self, len: usize) -> Result<(), Error> {
        if len > u8::MAX as usize {
            self.writer.write_u8(ERL_LARGE_TUPLE_EXT)?;
            self.writer.write_u32(len as u32)
        } else {
            self.writer.write_all(&[ERL_SMALL_TUPLE_EXT, len as u8])
        }
    }
}

#[rustfmt::skip]
macro_rules! compound {
    ($s: expr) => { Compound { ser: $s, nil: false } };
    ($s: expr, $b: expr) => { Compound { ser: $s, nil: $b } };
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let s = if v { "true" } else { "false" };
        self.writer.write_u8(ERL_SMALL_ATOM_UTF8_EXT)?;
        self.writer.write_u8(s.len() as u8)?;
        self.writer.write_all(s.as_bytes())
    }

    #[rustfmt::skip]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        match v {
            i if i >= 0 => self.serialize_u8(i as u8),
            i           => self.write_i27(i as i32),
        }
    }

    #[rustfmt::skip]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        match self.etype.pop() {
            None => {
                match v {
                    i if range!(i, u8, i16) => self.serialize_u8(i as u8),
                    i                       => self.write_i27(i as i32),
                }
            }
            Some([ERL_NEWER_REFERENCE_EXT, _]) if range!(v, 0, 5, i16) => {
                self.ref_n = if v > 0 { Some(v as usize) } else { None };
                self.writer.write_i16(v)
            }
            o => Err(interrupted!("serialize_i16: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        match v {
            i if range!(i,  u8, i32) => self.serialize_u8(i as u8),
            i if range!(i, i27, i32) => self.write_i27(i),
            i if i >= 0              => self.write_small_big(0, i as u64),
            i if i >  i32::MIN       => self.write_small_big(1, i.abs() as u64),
            i                        => Err(invalid_input!("serialize_i32: {}", i)),
        }
    }

    #[rustfmt::skip]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        match v {
            i if range!(i,  u8, i64) => self.serialize_u8(i as u8),
            i if range!(i, i27, i64) => self.write_i27(i as i32),
            i if i >= 0              => self.write_small_big(0, i as u64),
            i if i >  i64::MIN       => self.write_small_big(1, i.abs() as u64),
            i                        => Err(invalid_input!("serialize_i64: {}", i)),
        }
    }

    serde_if_integer128! {
        fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
            Err(unsupported!("serialize_i128"))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&[ERL_SMALL_INTEGER_EXT, v])
    }

    #[rustfmt::skip]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        match v {
            u if u <= u8::MAX as u16 => self.serialize_u8(u as u8),
            u                        => self.write_i27(u as i32),
        }
    }

    #[rustfmt::skip]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        match self.etype.pop() {
            None => {
                match v {
                    u if u <=  u8::MAX as u32 => self.serialize_u8(u as u8),
                    u if u <= i27::MAX as u32 => self.write_i27(u as i32),
                    u                         => self.write_small_big(0, u as u64),
                }
            }
            Some([ERL_NEW_PID_EXT, _]) | Some([ERL_NEW_PORT_EXT, _]) | Some([ERL_V4_PORT_EXT, _]) | Some([ERL_NEWER_REFERENCE_EXT, _]) => {
                self.writer.write_u32(v)
            }
            o => Err(interrupted!("serialize_u32: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        match self.etype.pop() {
            None => {
                match v {
                    u if u <=  u8::MAX as u64 => self.serialize_u8(u as u8),
                    u if u <= i27::MAX as u64 => self.write_i27(u as i32),
                    u                         => self.write_small_big(0, u),
                }
            }
            Some([ERL_V4_PORT_EXT, _]) => {
                self.writer.write_u64(v)
            }
            o => Err(interrupted!("serialize_u64: {:?}", o)),
        }
    }

    serde_if_integer128! {
        fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
            Err(unsupported!("serialize_u128"))
        }
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(unsupported!("serialize_f32"))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(NEW_FLOAT_EXT)?;
        self.writer.write_f64(v)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(unsupported!("serialize_char"))
    }

    #[rustfmt::skip]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        match (self.etype.pop(), v.len()) {
            (None, u) if u == u16::MIN as usize => {
                self.serialize_unit()
            }
            (None, u) if u <= u16::MAX as usize => {
                self.writer.write_u8(ERL_STRING_EXT)?;
                self.writer.write_u16(u as u16)?;
                self.writer.write_all(v.as_bytes())
            }
            (Some([ERL_ATOM_EXT, _]), u) if u <= MAXATOMLEN as usize => {
                self.writer.write_u8(ERL_ATOM_EXT)?;
                self.writer.write_u16(u as u16)?;
                self.writer.write_all(v.as_bytes())
            }
            (Some([ERL_SMALL_ATOM_UTF8_EXT, _]), u) if u <= u8::MAX as usize => {
                self.writer.write_u8(ERL_SMALL_ATOM_UTF8_EXT)?;
                self.writer.write_u8(u as u8)?;
                self.writer.write_all(v.as_bytes())
            }
            (Some([ERL_ATOM_UTF8_EXT, _]), u) if u <= MAXATOMLEN_UTF8 as usize => {
                self.writer.write_u8(ERL_ATOM_UTF8_EXT)?;
                self.writer.write_u16(u as u16)?;
                self.writer.write_all(v.as_bytes())
            }
            o => Err(interrupted!("serialize_str: {:?}", o)),
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match v.len() {
            u if u <= u32::MAX as usize => {
                self.writer.write_u8(ERL_BINARY_EXT)?;
                self.writer.write_u32(u as u32)?;
                self.writer.write_all(v)
            }
            u => Err(invalid_input!("serialize_bytes: {}", u)),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(ERL_NIL_EXT)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(unsupported!("serialize_unit_variant: {}", name))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        self.writer.write_all(&[ERL_SMALL_TUPLE_EXT, 1])?;
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        match (name, variant) {
            ("Atom", "Latin1") => {
                self.etype.push([ERL_ATOM_EXT, 0]);
                value.serialize(&mut *self)
            }
            ("Atom", "UTF8Small") => {
                self.etype.push([ERL_SMALL_ATOM_UTF8_EXT, 0]);
                value.serialize(&mut *self)
            }
            ("Atom", "UTF8") => {
                self.etype.push([ERL_ATOM_UTF8_EXT, 0]);
                value.serialize(&mut *self)
            }
            _ => Err(unsupported!("serialize_newtype_variant: {}", name)),
        }
    }

    #[rustfmt::skip]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, self::Error> {
        match self.etype.pop() {
            None => {
                match len {
                    Some(u) if u > usize::MIN => {
                        self.writer.write_u8(ERL_LIST_EXT)?;
                        self.writer.write_u32(u as u32)?;
                        Ok(compound!(self, true))
                    }
                    _ => Ok(compound!(self, true)),
                }
            }
            Some([ERL_NEWER_REFERENCE_EXT, 4]) => {
                if let Some(u) = self.ref_n {
                    self.etype
                        .append(&mut [[ERL_NEWER_REFERENCE_EXT, 4]].repeat(u));
                }
                Ok(compound!(self))
            }
            o => Err(interrupted!("serialize_seq: {:?}", o)),
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, self::Error> {
        self.write_tuple(len)?;
        Ok(compound!(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, self::Error> {
        self.write_tuple(len)?;
        Ok(compound!(self))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, self::Error> {
        Err(unsupported!("serialize_tuple_variant: {}", name))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, self::Error> {
        match len {
            Some(u) => {
                self.writer.write_u8(ERL_MAP_EXT)?;
                self.writer.write_u32(u as u32)?;
                Ok(compound!(self))
            }
            None => Ok(compound!(self)),
        }
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, self::Error> {
        match name {
            "Pid" => {
                self.writer.write_u8(ERL_NEW_PID_EXT)?;
                self.etype.push([ERL_NEW_PID_EXT, 4]); // creation
                self.etype.push([ERL_NEW_PID_EXT, 3]); // serial
                self.etype.push([ERL_NEW_PID_EXT, 2]); // num
                Ok(compound!(self))
            }
            "Ref" => {
                self.writer.write_u8(ERL_NEWER_REFERENCE_EXT)?;
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 4]); // n
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 3]); // creation
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 1]); // len
                Ok(compound!(self))
            }
            _ => {
                self.write_tuple(len)?;
                Ok(compound!(self))
            }
        }
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, self::Error> {
        match (name, variant) {
            ("Port", "NewPort") => {
                self.writer.write_u8(ERL_NEW_PORT_EXT)?;
                self.etype.push([ERL_NEW_PORT_EXT, 3]); // creation
                self.etype.push([ERL_NEW_PORT_EXT, 2]); // id
                Ok(compound!(self))
            }
            ("Port", "V4Port") => {
                self.writer.write_u8(ERL_V4_PORT_EXT)?;
                self.etype.push([ERL_V4_PORT_EXT, 3]); // creation
                self.etype.push([ERL_V4_PORT_EXT, 2]); // id
                Ok(compound!(self))
            }
            ("Msg", "Send") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_SEND,
                ])?;
                Ok(compound!(self))
            }
            ("Msg", "SendTT") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_SEND_TT,
                ])?;
                Ok(compound!(self))
            }
            ("Msg", "RegSend") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_REG_SEND,
                ])?;
                Ok(compound!(self))
            }
            ("Msg", "RegSendTT") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_REG_SEND_TT,
                ])?;
                Ok(compound!(self))
            }
            ("Msg", "Exit") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_EXIT,
                ])?;
                Ok(compound!(self))
            }
            ("Msg", "ExitTT") => {
                self.writer.write_all(&[
                    ERL_SMALL_TUPLE_EXT,
                    len as u8 + 1,
                    ERL_SMALL_INTEGER_EXT,
                    ERL_EXIT_TT,
                ])?;
                Ok(compound!(self))
            }
            _ => Err(unsupported!("serialize_struct_variant: {}", name)),
        }
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct Compound<'a, W> {
    ser: &'a mut Serializer<W>,
    nil: bool,
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.nil {
            self.ser.writer.write_u8(ERL_NIL_EXT)
        } else {
            Ok(())
        }
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<K>(&mut self, key: &K) -> Result<Self::Ok, Self::Error>
    where
        K: ser::Serialize + ?Sized,
    {
        key.serialize(&mut *self.ser)
    }

    fn serialize_value<V>(&mut self, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: io::Write,
    T: ser::Serialize + ?Sized,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

pub fn to_vec<T>(value: &T, capacity: usize) -> Result<Vec<u8>, Error>
where
    T: ser::Serialize + ?Sized,
{
    let mut vec = Vec::with_capacity(capacity);
    to_writer(&mut vec, value).and_then(|()| Ok(vec))
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {

    use std::collections::HashMap;
    use std::ffi::CString;

    use serde::Serialize;

    use crate::i27;
    use crate::term;

    #[derive(Serialize)]
    struct Color {
        r: i64,
        g: u64,
        b: f64,
    }

    #[derive(Serialize)]
    struct Point2D(i8, u8);

    #[derive(Serialize)]
    struct Inches(String);

    #[derive(Serialize)]
    struct Instance;

    macro_rules! test {
        ($value: expr) => {
            super::to_vec(&$value, 80)
        };
    }

    #[test]
    fn serialize_bool() {
        for (input, expected) in vec![
            (true,  vec![0x77, 0x04, 0x74, 0x72, 0x75, 0x65]),
            (false, vec![0x77, 0x05, 0x66, 0x61, 0x6c, 0x73, 0x65]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_i8() {
        for (input, expected) in vec![
            (i8::MIN,           vec![0x62, 0xff, 0xff, 0xff, 0x80]),
            (u8::MIN as i8 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            (u8::MIN as i8,     vec![0x61, 0x00]),
            (u8::MIN as i8 + 1, vec![0x61, 0x01]),
            (i8::MAX,           vec![0x61, 0x7f]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_i16() {
        for (input, expected) in vec![
            (i16::MIN,            vec![0x62, 0xff, 0xff, 0x80, 0x00]),
            ( u8::MIN as i16 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            ( u8::MIN as i16,     vec![0x61, 0x00]),
            ( u8::MIN as i16 + 1, vec![0x61, 0x01]),
            ( u8::MAX as i16,     vec![0x61, 0xff]),
            ( u8::MAX as i16 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i16::MAX,            vec![0x62, 0x00, 0x00, 0x7f, 0xff]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_i32() {
        for (input, expected) in vec![
            (i32::MIN        + 1, vec![0x6e, 0x04, 0x01, 0xff, 0xff, 0xff, 0x7f]),
            (i27::MIN as i32 - 1, vec![0x6e, 0x04, 0x01, 0x01, 0x00, 0x00, 0x08]),
            (i27::MIN as i32,     vec![0x62, 0xf8, 0x00, 0x00, 0x00]),
            ( u8::MIN as i32 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            ( u8::MIN as i32,     vec![0x61, 0x00]),
            ( u8::MIN as i32 + 1, vec![0x61, 0x01]),
            ( u8::MAX as i32,     vec![0x61, 0xff]),
            ( u8::MAX as i32 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as i32,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as i32 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (i32::MAX,            vec![0x6e, 0x04, 0x00, 0xff, 0xff, 0xff, 0x7f]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
        for (input, _expected) in vec![
            (i32::MIN, vec![0x6e, 0x04, 0x01, 0x00, 0x00, 0x00, 0x80]),
        ] {
            assert!(test!(&input).is_err(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_i64() {
        for (input, expected) in vec![
            (i64::MIN        + 1, vec![0x6e, 0x08, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]),
            (i27::MIN as i64 - 1, vec![0x6e, 0x04, 0x01, 0x01, 0x00, 0x00, 0x08]),
            (i27::MIN as i64,     vec![0x62, 0xf8, 0x00, 0x00, 0x00]),
            ( u8::MIN as i64 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            ( u8::MIN as i64,     vec![0x61, 0x00]),
            ( u8::MIN as i64 + 1, vec![0x61, 0x01]),
            ( u8::MAX as i64,     vec![0x61, 0xff]),
            ( u8::MAX as i64 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as i64,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as i64 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (i64::MAX,            vec![0x6e, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
        for (input, _expected) in vec![
            (i64::MIN, vec![0x6e, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ] {
            assert!(test!(&input).is_err(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_u8() {
        for (input, expected) in vec![
            (u8::MIN,     vec![0x61, 0x00]),
            (u8::MIN + 1, vec![0x61, 0x01]),
            (u8::MAX,     vec![0x61, 0xff]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_u16() {
        for (input, expected) in vec![
            ( u8::MIN as u16,     vec![0x61, 0x00]),
            ( u8::MIN as u16 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u16,     vec![0x61, 0xff]),
            ( u8::MAX as u16 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (u16::MAX,            vec![0x62, 0x00, 0x00, 0xff, 0xff]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_u32() {
        for (input, expected) in vec![
            ( u8::MIN as u32,     vec![0x61, 0x00]),
            ( u8::MIN as u32 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u32,     vec![0x61, 0xff]),
            ( u8::MAX as u32 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as u32,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as u32 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (u32::MAX,            vec![0x6e, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_u64() {
        for (input, expected) in vec![
            ( u8::MIN as u64,     vec![0x61, 0x00]),
            ( u8::MIN as u64 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u64,     vec![0x61, 0xff]),
            ( u8::MAX as u64 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as u64,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as u64 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (u64::MAX,            vec![0x6e, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_f64() {
        for (input, expected) in vec![
            (-1.0 as f64, vec![0x46, 0xbf, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (-0.0 as f64, vec![0x46, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            ( 0.0 as f64, vec![0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            ( 1.0 as f64, vec![0x46, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_str() {
        for (input, expected) in vec![
            ("",      vec![0x6a]),
            ("hello", vec![0x6b, 0x00, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_bytes() {
        for (input, expected) in vec![
            ("",      vec![0x6d, 0x00, 0x00, 0x00, 0x00]),
            ("hello", vec![0x6d, 0x00, 0x00, 0x00, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        ] {
            let v = CString::new(input).unwrap();
            assert_eq!(expected, test!(&v).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_unit_struct() {
        assert_eq!(vec![0x6a], test!(Instance {}).unwrap());
    }

    #[test]
    fn serialize_newtype_struct() {
        assert_eq!(
            vec![
                0x68, 0x01,
                0x6b, 0x00, 0x01, 0x61
            ],
            test!(Inches("a".to_owned())).unwrap()
        );
    }

    #[test]
    fn serialize_seq() {
        for (input, expected) in vec![
            (vec![],  vec![0x6a]),
            (vec![1], vec![0x6c, 0x00, 0x00, 0x00, 0x01, 0x61, 0x01, 0x6a]),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_tuple() {
        assert_eq!(
            vec![
                0x6a
            ],
            test!(()).unwrap()
        );

        assert_eq!(
            vec![
                0x68, 0x01,
                0x61, 0x01
            ], test!((1,)).unwrap()
        );

        assert_eq!(
            vec![
                0x68, 0x02,
                0x61, 0x01,
                0x77, 0x04, 0x74, 0x72, 0x75, 0x65
            ],
            test!((1, true)).unwrap()
        );
    }

    #[test]
    fn serialize_tuple_struct() {
        assert_eq!(
            vec![
                0x68, 0x02,
                0x61, 0x01,
                0x61, 0x02
            ],
            test!(Point2D(1, 2)).unwrap()
        );
    }

    #[test]
    fn serialize_map() {
        let mut map = HashMap::new();

        map.clear();
        assert_eq!(
            vec![
                0x74, 0x00, 0x00, 0x00, 0x00
            ],
            test!(&map).unwrap()
        );

        map.clear();
        map.insert(1, 2);
        assert_eq!(
            vec![
                0x74, 0x00, 0x00, 0x00, 0x01,
                0x61, 0x01,
                0x61, 0x02
            ],
            test!(&map).unwrap()
        );
    }

    #[test]
    fn serialize_struct() {
        assert_eq!(
            vec![
                0x68, 0x03,
                0x61, 0x01,
                0x61, 0x02,
                0x46, 0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ],
            test!(Color { r: 1, g: 2, b: 3 as f64}).unwrap()
        );
    }


    #[test]
    fn serialize_atom() {
        for (input, expected) in vec![
            (
                term::Atom::Latin1("n".to_owned()),
                vec![0x64, 0x00, 0x01, 0x6e],
            ),
            (
                term::Atom::UTF8Small("n".to_owned()),
                vec![0x77, 0x01, 0x6e],
            ),
            (
                term::Atom::UTF8("n".to_owned()),
                vec![0x76, 0x00, 0x01, 0x6e],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_pid() {
        for (input, expected) in vec![
            (
                term::Pid {
                    node: term::Atom::UTF8("n".to_owned()),
                    num: 1,
                    serial: 2,
                    creation: 3,
                },
                vec![
                    0x58,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x02,
                    0x00, 0x00, 0x00, 0x03,
                ],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_port() {
        for (input, expected) in vec![
            (
                term::Port::NewPort {
                    node: term::Atom::UTF8("n".to_owned()),
                    id: 1,
                    creation: 2,
                },
                vec![
                    0x59,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x02,
                ],
            ),
            (
                term::Port::V4Port {
                    node: term::Atom::UTF8("n".to_owned()),
                    id: 1,
                    creation: 2,
                },
                vec![
                    0x78,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x02,
                ],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_ref() {
        for (input, expected) in vec![
            (
                term::Ref {
                    len: 0,
                    node: term::Atom::UTF8("n".to_owned()),
                    creation: 1,
                    n: None,
                },
                vec![
                    0x5a,
                    0x00, 0x00,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x01,
                ],
            ),
            (
                term::Ref {
                    len: 1,
                    node: term::Atom::UTF8("n".to_owned()),
                    creation: 1,
                    n: Some(vec![2]),
                },
                vec![
                    0x5a,
                    0x00, 0x01,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x02,
                ],
            ),
            (
                term::Ref {
                    len: 5,
                    node: term::Atom::UTF8("n".to_owned()),
                    creation: 1,
                    n: Some(vec![2, 3, 4, 5, 6]),
                },
                vec![
                    0x5a,
                    0x00, 0x05,
                    0x76, 0x00, 0x01, 0x6e,
                    0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x06,
                ],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_trace() {
        for (input, expected) in vec![
            (
                term::Trace(
                    1,
                    2,
                    3,
                    term::Pid {
                        node: term::Atom::UTF8("n".to_owned()),
                        num: 4,
                        serial: 5,
                        creation: 6,
                    },
                    7,
                ),
                vec![
                    0x68, 0x05,
                    0x61, 0x01,
                    0x61, 0x02,
                    0x61, 0x03,
                    0x58,
                      0x76, 0x00, 0x01, 0x6e,
                      0x00, 0x00, 0x00, 0x04,
                      0x00, 0x00, 0x00, 0x05,
                      0x00, 0x00, 0x00, 0x06,
                    0x61, 0x07,
                ],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn serialize_msg() {
        for (input, expected) in vec![
            (
                term::Msg::Send {
                    cookie: term::Atom::UTF8("c".to_owned()),
                    to: term::Pid {
                        node: term::Atom::UTF8("n".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                },
                vec![
                    0x68, 0x03,
                    0x61, 0x02,
                    0x76, 0x00, 0x01, 0x63,
                    0x58,
                      0x76, 0x00, 0x01, 0x6e,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                ],
            ),
            (
                term::Msg::SendTT {
                    cookie: term::Atom::UTF8("c".to_owned()),
                    to: term::Pid {
                        node: term::Atom::UTF8("n1".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                    token: term::Trace(
                        4,
                        5,
                        6,
                        term::Pid {
                            node: term::Atom::UTF8("n2".to_owned()),
                            num: 7,
                            serial: 8,
                            creation: 9,
                        },
                        10,
                    ),
                },
                vec![
                    0x68, 0x04,
                    0x61, 0x0c,
                    0x76, 0x00, 0x01, 0x63,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x31,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                    0x68, 0x05,
                      0x61, 0x04,
                      0x61, 0x05,
                      0x61, 0x06,
                      0x58,
                        0x76, 0x00, 0x02, 0x6e, 0x32,
                        0x00, 0x00, 0x00, 0x07,
                        0x00, 0x00, 0x00, 0x08,
                        0x00, 0x00, 0x00, 0x09,
                      0x61, 0x0a,
                ],
            ),
            (
                term::Msg::RegSend {
                    from: term::Pid {
                        node: term::Atom::UTF8("n".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                    cookie: term::Atom::UTF8("c".to_owned()),
                    toname: term::Atom::UTF8("s".to_owned()),
                },
                vec![
                    0x68, 0x04,
                    0x61, 0x06,
                    0x58,
                      0x76, 0x00, 0x01, 0x6e,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                    0x76, 0x00, 0x01, 0x63,
                    0x76, 0x00, 0x01, 0x73,
                ],
            ),
            (
                term::Msg::RegSendTT {
                    from: term::Pid {
                        node: term::Atom::UTF8("n1".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                    cookie: term::Atom::UTF8("c".to_owned()),
                    toname: term::Atom::UTF8("s".to_owned()),
                    token: term::Trace(
                        4,
                        5,
                        6,
                        term::Pid {
                            node: term::Atom::UTF8("n2".to_owned()),
                            num: 7,
                            serial: 8,
                            creation: 9,
                        },
                        10,
                    ),
                },
                vec![
                    0x68, 0x05,
                    0x61, 0x10,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x31,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                    0x76, 0x00, 0x01, 0x63,
                    0x76, 0x00, 0x01, 0x73,
                    0x68, 0x05,
                      0x61, 0x04,
                      0x61, 0x05,
                      0x61, 0x06,
                      0x58,
                        0x76, 0x00, 0x02, 0x6e, 0x32,
                        0x00, 0x00, 0x00, 0x07,
                        0x00, 0x00, 0x00, 0x08,
                        0x00, 0x00, 0x00, 0x09,
                      0x61, 0x0a,
                ],
            ),
            (
                term::Msg::Exit {
                    from: term::Pid {
                        node: term::Atom::UTF8("n1".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                    to: term::Pid {
                        node: term::Atom::UTF8("n2".to_owned()),
                        num: 4,
                        serial: 5,
                        creation: 6,
                    },
                    reason: "any".to_owned(),
                },
                vec![
                    0x68, 0x04,
                    0x61, 0x03,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x31,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x32,
                      0x00, 0x00, 0x00, 0x04,
                      0x00, 0x00, 0x00, 0x05,
                      0x00, 0x00, 0x00, 0x06,
                    0x6b, 0x00, 0x03, 0x61, 0x6e, 0x79,
                ],
            ),
            (
                term::Msg::ExitTT {
                    from: term::Pid {
                        node: term::Atom::UTF8("n1".to_owned()),
                        num: 1,
                        serial: 2,
                        creation: 3,
                    },
                    to: term::Pid {
                        node: term::Atom::UTF8("n2".to_owned()),
                        num: 4,
                        serial: 5,
                        creation: 6,
                    },
                    token: term::Trace(
                        7,
                        8,
                        9,
                        term::Pid {
                            node: term::Atom::UTF8("n3".to_owned()),
                            num: 10,
                            serial: 11,
                            creation: 12,
                        },
                        13,
                    ),
                    reason: "any".to_owned(),
                },
                vec![
                    0x68, 0x05,
                    0x61, 0x0d,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x31,
                      0x00, 0x00, 0x00, 0x01,
                      0x00, 0x00, 0x00, 0x02,
                      0x00, 0x00, 0x00, 0x03,
                    0x58,
                      0x76, 0x00, 0x02, 0x6e, 0x32,
                      0x00, 0x00, 0x00, 0x04,
                      0x00, 0x00, 0x00, 0x05,
                      0x00, 0x00, 0x00, 0x06,
                    0x68, 0x05,
                      0x61, 0x07,
                      0x61, 0x08,
                      0x61, 0x09,
                      0x58,
                        0x76, 0x00, 0x02, 0x6e, 0x33,
                        0x00, 0x00, 0x00, 0x0a,
                        0x00, 0x00, 0x00, 0x0b,
                        0x00, 0x00, 0x00, 0x0c,
                      0x61, 0x0d,
                    0x6b, 0x00, 0x03, 0x61, 0x6e, 0x79,
                ],
            ),
        ] {
            assert_eq!(expected, test!(&input).unwrap(), "{:?}", input);
        }
    }
}
