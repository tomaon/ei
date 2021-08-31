use std::io;

use serde::{de, serde_if_integer128};

use crate::consts::*;
use crate::error::Error;
use crate::io::{Number, Reader};

pub struct Deserializer<R> {
    reader: Reader<R>,
    etype: Vec<[u8; 2]>,
    ref_n: Option<usize>,
}

impl<R> Deserializer<R>
where
    R: io::Read,
{
    pub fn new(r: R) -> Self {
        Deserializer {
            reader: Reader::new(r),
            etype: Vec::with_capacity(16),
            ref_n: None,
        }
    }

    pub fn read_number(&mut self) -> Result<Number, Error> {
        match self.reader.read_u8()? {
            ERL_SMALL_INTEGER_EXT => self.reader.read_u8().map(Number::U8),
            ERL_INTEGER_EXT => self.reader.read_i32().map(Number::I32),
            ERL_SMALL_BIG_EXT => self.read_small_big(),
            u => Err(invalid_data!("read_number: {}", u)),
        }
    }

    fn read_small_big(&mut self) -> Result<Number, Error> {
        match self.reader.read_u8()? {
            a if a <= 8 => {
                let s = self.reader.read_u8()?;
                let mut n = 0u64;
                for i in 0..a {
                    let u = self.reader.read_u8()?;
                    n |= (u as u64) << (i * 8);
                }
                Ok(Number::SmallBig(n, s))
            }
            a => Err(invalid_data!("read_small_big: {}", a)),
        }
    }

    #[rustfmt::skip]
    fn read_tuple(&mut self) -> Result<usize, Error> {
        match self.reader.read_u8()? {
            ERL_SMALL_TUPLE_EXT => {
                self.reader.read_u8().map(|u| u as usize)
            }
            ERL_LARGE_TUPLE_EXT => {
                self.reader.read_u32().map(|u| u as usize)
            }
            ERL_NEW_PID_EXT => {
                self.etype.push([ERL_NEW_PID_EXT, 4]); // creation
                self.etype.push([ERL_NEW_PID_EXT, 3]); // serial
                self.etype.push([ERL_NEW_PID_EXT, 2]); // num
                Ok(4)
            }
            ERL_NEWER_REFERENCE_EXT => {
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 4]); // n
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 3]); // creation
                self.etype.push([ERL_NEWER_REFERENCE_EXT, 1]); // len
                Ok(4)
            }
            u => Err(invalid_data!("read_tuple: {}", u)),
        }
    }

    pub fn read_unit(&mut self) -> Result<(), Error> {
        match self.reader.read_u8()? {
            ERL_NIL_EXT => Ok(()),
            u => Err(invalid_data!("read_unit: {}", u)),
        }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: io::Read,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("deserialize_any")
    }

    #[rustfmt::skip]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.read_u8()? {
            ERL_ATOM_EXT | ERL_ATOM_UTF8_EXT => {
                self.reader.read_string_u16().and_then(|s| visitor.visit_bool(s == "true"))
            }
            ERL_SMALL_ATOM_UTF8_EXT => {
                self.reader.read_string_u8().and_then(|s| visitor.visit_bool(s == "true"))
            }
            u => Err(invalid_data!("deserialize_bool: {}", u)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_number()? {
            Number::U8(u)  if u <= i8::MAX as u8 => visitor.visit_i8(u as i8),
            Number::I32(i) if range!(i, i8, i32) => visitor.visit_i8(i as i8),
            e                                    => Err(invalid_data!("deserialize_i8: {:?}", e)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.etype.pop() {
            None => {
                match self.read_number()? {
                    Number::U8(u)                         => visitor.visit_i16(u as i16),
                    Number::I32(i) if range!(i, i16, i32) => visitor.visit_i16(i as i16),
                    e                                     => Err(invalid_data!("deserialize_i16: {:?}", e)),
                }
            }
            Some([ERL_NEWER_REFERENCE_EXT, _]) => {
                let i = self.reader.read_i16()?;
                self.ref_n = if i > 0 { Some(i as usize) } else { None };
                visitor.visit_i16(i)
            }
            o => Err(interrupted!("deserialize_i16: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_number()? {
            Number::U8(u)                                  => visitor.visit_i32(u as i32),
            Number::I32(i)                                 => visitor.visit_i32(i),
            Number::SmallBig(u, 0) if u <= i32::MAX as u64 => visitor.visit_i32(u as i32),
            Number::SmallBig(u, 1) if u <= i32::MAX as u64 => visitor.visit_i32(u as i32 * -1),
            e                                              => Err(invalid_data!("deserialize_i32: {:?}", e)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_number()? {
            Number::U8(u)                                  => visitor.visit_i64(u as i64),
            Number::I32(i)                                 => visitor.visit_i64(i as i64),
            Number::SmallBig(u, 0) if u <= i64::MAX as u64 => visitor.visit_i64(u as i64),
            Number::SmallBig(u, 1) if u <= i64::MAX as u64 => visitor.visit_i64(u as i64 * -1),
            e                                              => Err(invalid_data!("deserialize_i64: {:?}", e)),
        }
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>
        {
            Err(unsupported!("deserialize_i128"))
        }
    }

    #[rustfmt::skip]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_number()? {
            Number::U8(u) => visitor.visit_u8(u),
            e             => Err(invalid_data!("deserialize_u8: {:?}", e)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_number()? {
            Number::U8(u)                         => visitor.visit_u16(u as u16),
            Number::I32(i) if range!(i, u16, i32) => visitor.visit_u16(i as u16),
            e                                     => Err(invalid_data!("deserialize_u16: {:?}", e)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.etype.pop() {
            None => {
                match self.read_number()? {
                    Number::U8(u)                                  => visitor.visit_u32(u as u32),
                    Number::I32(i)         if i >= 0               => visitor.visit_u32(i as u32),
                    Number::SmallBig(u, 0) if u <= u32::MAX as u64 => visitor.visit_u32(u as u32),
                    e                                              => Err(invalid_data!("deserialize_u32: {:?}", e)),
                }
            }
            Some([ERL_NEW_PID_EXT, _]) | Some([ERL_NEW_PORT_EXT, _]) | Some([ERL_V4_PORT_EXT, _]) | Some([ERL_NEWER_REFERENCE_EXT,_]) => {
                self.reader.read_u32().and_then(|u| visitor.visit_u32(u))
            }
            o => Err(interrupted!("deserialize_u32: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.etype.pop() {
            None => {
                match self.read_number()? {
                    Number::U8(u)                    => visitor.visit_u64(u as u64),
                    Number::I32(i)         if i >= 0 => visitor.visit_u64(i as u64),
                    Number::SmallBig(u, 0)           => visitor.visit_u64(u),
                    e                                => Err(invalid_data!("deserialize_u64: {:?}", e)),
                }
            }
            Some([ERL_V4_PORT_EXT, _]) => {
                self.reader.read_u64().and_then(|u| visitor.visit_u64(u))
            }
            o => Err(interrupted!("deserialize_u64: {:?}", o)),
        }
    }

    serde_if_integer128! {
        fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>
        {
            Err(unsupported!("deserialize_u128"))
        }
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(unsupported!("deserialize_f32"))
    }

    #[rustfmt::skip]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.read_u8()? {
            NEW_FLOAT_EXT => self.reader.read_f64().and_then(|f| visitor.visit_f64(f)),
            u             => Err(invalid_data!("deserialize_f64: {}", u)),
        }
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(unsupported!("deserialize_char"))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(unsupported!("deserialize_str"))
    }

    #[rustfmt::skip]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.etype.pop() {
            None => {
                match self.reader.read_u8()? {
                    ERL_NIL_EXT => {
                        visitor.visit_str("")
                    }
                    ERL_STRING_EXT => {
                        self.reader.read_string_u16().and_then(|s| visitor.visit_string(s))
                    }
                    u => Err(invalid_data!("deserialize_string: {}", u)),
                }
            }
            Some([ERL_ATOM_EXT, _]) | Some([ERL_ATOM_UTF8_EXT, _]) => {
                self.reader.read_string_u16().and_then(|s| visitor.visit_string(s))
            }
            Some([ERL_SMALL_ATOM_UTF8_EXT, _]) => {
                self.reader.read_string_u8().and_then(|s| visitor.visit_string(s))
            }
            o => Err(interrupted!("deserialize_string: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.read_u8()? {
            ERL_BINARY_EXT => {
                self.reader.read_exact_u32().and_then(|v| visitor.visit_bytes(&v))
            }
            u => Err(invalid_data!("deserialize_bytes: {}", u)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.ref_n {
            Some(_) => visitor.visit_some(self),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.read_unit().and_then(|()| visitor.visit_unit())
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_tuple()? {
            1 => visitor.visit_newtype_struct(self),
            u => Err(interrupted!("deserialize_newtype_struct: {}, {}", name, u)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.etype.pop() {
            None => {
                match self.reader.read_u8()? {
                    ERL_NIL_EXT => {
                        visitor.visit_seq(ListAccess {
                            de: self,
                            len: None,
                        })
                    },
                    ERL_LIST_EXT => {
                        let u = self.reader.read_u32()? as usize;
                        visitor.visit_seq(ListAccess {
                            de: self,
                            len: Some(u),
                        })
                    }
                    u => Err(invalid_data!("deserialize_seq: {}", u)),
                }
            }
            Some([ERL_NEWER_REFERENCE_EXT, 4]) => {
                match self.ref_n {
                    Some(u) => {
                        self.etype.append(&mut [[ERL_NEWER_REFERENCE_EXT, 4]].repeat(u + 1));
                        visitor.visit_seq(ArrayAccess {
                            de: self,
                            len: Some(u),
                        })
                    }
                    None => {
                        visitor.visit_seq(ArrayAccess {
                            de: self,
                            len: None,
                        })
                    }
                }
            }
            o => Err(interrupted!("deserialize_seq: {:?}", o)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_tuple()? {
            u if u == len => {
                visitor.visit_seq(ListAccess {
                    de: self,
                    len: Some(u),
                })
            }
            u => Err(interrupted!("deserialize_tuple: {}", u)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_tuple()? {
            u if u == len => {
                visitor.visit_seq(ListAccess {
                    de: self,
                    len: Some(u),
                })
            }
            u => Err(interrupted!("deserialize_tuple_struct: {}, {}", name, u)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.read_u8()? {
            ERL_MAP_EXT => {
                match self.reader.read_u32()? as usize {
                    0 => {
                        visitor.visit_map(MapAccess {
                            de: self,
                            len: None,
                        })
                    }
                    u => {
                        visitor.visit_map(MapAccess {
                            de: self,
                            len: Some(u),
                        })
                    }
                }
            }
            u => Err(invalid_data!("deserialize_map: {}", u)),
        }
    }

    #[rustfmt::skip]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match name.len() {
            0 => {
                visitor.visit_seq(ListAccess {
                    de: self,
                    len: Some(fields.len()),
                })
            }
            _ => match self.read_tuple()? {
                u if u == fields.len() => {
                    visitor.visit_seq(ListAccess {
                        de: self,
                        len: Some(u),
                    })
                }
                u => Err(interrupted!("deserialize_struct: {}, {}", name, u)),
            },
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    #[rustfmt::skip]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.read_u8()? {
            ERL_ATOM_EXT => {
                self.etype.push([ERL_ATOM_EXT, 0]);
                visitor.visit_str("Latin1")
            }
            ERL_SMALL_ATOM_UTF8_EXT => {
                self.etype.push([ERL_SMALL_ATOM_UTF8_EXT, 0]);
                visitor.visit_str("UTF8Small")
            }
            ERL_ATOM_UTF8_EXT => {
                self.etype.push([ERL_ATOM_UTF8_EXT, 0]);
                visitor.visit_str("UTF8")
            }
            ERL_NEW_PORT_EXT => {
                self.etype.push([ERL_NEW_PORT_EXT, 3]); // creation
                self.etype.push([ERL_NEW_PORT_EXT, 2]); // id
                visitor.visit_str("NewPort")
            }
            ERL_V4_PORT_EXT => {
                self.etype.push([ERL_V4_PORT_EXT, 3]); // creation
                self.etype.push([ERL_V4_PORT_EXT, 2]); // id
                visitor.visit_str("V4Port")
            }
            ERL_SMALL_TUPLE_EXT => {
                match self.reader.read_exact_const::<3>()? {
                    [0x03, ERL_SMALL_INTEGER_EXT, ERL_SEND]        => visitor.visit_str("Send"),
                    [0x04, ERL_SMALL_INTEGER_EXT, ERL_SEND_TT]     => visitor.visit_str("SendTT"),
                    [0x04, ERL_SMALL_INTEGER_EXT, ERL_REG_SEND]    => visitor.visit_str("RegSend"),
                    [0x05, ERL_SMALL_INTEGER_EXT, ERL_REG_SEND_TT] => visitor.visit_str("RegSendTT"),
                    [0x04, ERL_SMALL_INTEGER_EXT, ERL_EXIT]        => visitor.visit_str("Exit"),
                    [0x05, ERL_SMALL_INTEGER_EXT, ERL_EXIT_TT]     => visitor.visit_str("ExitTT"),
                    v                                              => Err(interrupted!("deserialize_identifier: {:?}", v)),
                }
            }
            u => Err(invalid_data!("deserialize_identifier: {}", u)),
        }
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("deserialize_ignored_any");
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de, 'a, R> de::EnumAccess<'de> for &'a mut Deserializer<R>
where
    R: io::Read,
{
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self).map(|v| (v, self))
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for &'a mut Deserializer<R>
where
    R: io::Read,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        unimplemented!("unit_variant")
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("tuple_variant")
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}

struct ListAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    len: Option<usize>,
}

impl<'de, 'a, R> de::SeqAccess<'de> for ListAccess<'a, R>
where
    R: io::Read,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.len {
            Some(0) => {
                self.len = None;
                self.de.read_unit().map(|()| None)
            }
            Some(u) => {
                self.len = Some(u - 1);
                seed.deserialize(&mut *self.de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct ArrayAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    len: Option<usize>,
}

impl<'de, 'a, R> de::SeqAccess<'de> for ArrayAccess<'a, R>
where
    R: io::Read,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.len {
            Some(u) => {
                self.len = if u > 1 { Some(u - 1) } else { None };
                seed.deserialize(&mut *self.de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    len: Option<usize>,
}

impl<'de, 'a, R> de::MapAccess<'de> for MapAccess<'a, R>
where
    R: io::Read,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.len {
            Some(u) => {
                self.len = if u > 1 { Some(u - 1) } else { None };
                seed.deserialize(&mut *self.de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: io::Read,
    T: de::DeserializeOwned,
{
    let mut de = Deserializer::new(reader);
    de::Deserialize::deserialize(&mut de)
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {

    use std::collections::HashMap;
    use std::ffi::CString;

    use serde::Deserialize;

    use crate::error::Error;
    use crate::i27;
    use crate::term;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Color {
        r: i64,
        g: u64,
        b: f64,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Point2D(i8, u8);

    #[derive(Deserialize, PartialEq, Debug)]
    struct Inches(String);

    #[derive(Deserialize, PartialEq, Debug)]
    struct Instance;

    macro_rules! test {
        ($value: expr) => {
            super::from_reader($value.as_slice())
        };
    }

    #[test]
    fn deserialize_bool() {
        for (expected, input) in vec![
            (true,  vec![0x64, 0x00, 0x04, 0x74, 0x72, 0x75, 0x65]),
            (true,  vec![0x76, 0x00, 0x04, 0x74, 0x72, 0x75, 0x65]),
            (false, vec![0x77, 0x05, 0x66, 0x61, 0x6c, 0x73, 0x65]),
        ] {
            let actual: Result<bool, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_i8() {
        for (expected, input) in vec![
            (i8::MIN,           vec![0x62, 0xff, 0xff, 0xff, 0x80]),
            (u8::MIN as i8 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            (u8::MIN as i8,     vec![0x61, 0x00]),
            (u8::MIN as i8 + 1, vec![0x61, 0x01]),
            (i8::MAX,           vec![0x61, 0x7f]),
        ] {
            let actual: Result<i8, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_i16() {
        for (expected, input) in vec![
            (i16::MIN,            vec![0x62, 0xff, 0xff, 0x80, 0x00]),
            ( u8::MIN as i16 - 1, vec![0x62, 0xff, 0xff, 0xff, 0xff]),
            ( u8::MIN as i16,     vec![0x61, 0x00]),
            ( u8::MIN as i16 + 1, vec![0x61, 0x01]),
            ( u8::MAX as i16,     vec![0x61, 0xff]),
            ( u8::MAX as i16 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i16::MAX,            vec![0x62, 0x00, 0x00, 0x7f, 0xff]),
        ] {
            let actual: Result<i16, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_i32() {
        for (expected, input) in vec![
            // ei
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
            // term_to_binary
            (i32::MIN,            vec![0x62, 0x80, 0x00, 0x00, 0x00]),
            (i27::MIN as i32 - 1, vec![0x62, 0xF7, 0xFF, 0xFF, 0xFF]),
            (i27::MAX as i32 + 1, vec![0x62, 0x08, 0x00, 0x00, 0x00]),
            (i32::MAX,            vec![0x62, 0x7F, 0xFF, 0xFF, 0xFF]),
        ] {
            let actual: Result<i32, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
        for (_expected, input) in vec![
            (i32::MIN, vec![0x6e, 0x04, 0x01, 0x00, 0x00, 0x00, 0x80]),
        ] {
            let actual: Result<i32, Error> = test!(&input);
            assert!(actual.is_err(), "{:?}", actual);
        }
    }

    #[test]
    fn deserialize_i64() {
        for (expected, input) in vec![
            // ei
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
            // term_to_binary
            (i32::MIN as i64 - 1, vec![0x6E, 0x04, 0x01, 0x01, 0x00, 0x00, 0x80]),
            (i32::MIN as i64,     vec![0x62, 0x80, 0x00, 0x00, 0x00]),
            (i27::MIN as i64 - 1, vec![0x62, 0xF7, 0xFF, 0xFF, 0xFF]),
            (i27::MAX as i64 + 1, vec![0x62, 0x08, 0x00, 0x00, 0x00]),
            (i32::MAX as i64,     vec![0x62, 0x7F, 0xFF, 0xFF, 0xFF]),
            (i32::MAX as i64 + 1, vec![0x6E, 0x04, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ] {
            let actual: Result<i64, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
        for (_expected, input) in vec![
            (i64::MIN, vec![0x6e, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ] {
            let actual: Result<i64, Error> = test!(&input);
            assert!(actual.is_err(), "{:?}", actual);
        }
    }

    #[test]
    fn deserialize_u8() {
        for (expected, input) in vec![
            (u8::MIN,     vec![0x61, 0x00]),
            (u8::MIN + 1, vec![0x61, 0x01]),
            (u8::MAX,     vec![0x61, 0xff]),
        ] {
            let actual: Result<u8, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_u16() {
        for (expected, input) in vec![
            ( u8::MIN as u16,     vec![0x61, 0x00]),
            ( u8::MIN as u16 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u16,     vec![0x61, 0xff]),
            ( u8::MAX as u16 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (u16::MAX,            vec![0x62, 0x00, 0x00, 0xff, 0xff]),
        ] {
            let actual: Result<u16, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_u32() {
        for (expected, input) in vec![
            // ei
            ( u8::MIN as u32,     vec![0x61, 0x00]),
            ( u8::MIN as u32 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u32,     vec![0x61, 0xff]),
            ( u8::MAX as u32 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as u32,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as u32 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (u32::MAX,            vec![0x6e, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff]),
            // term_to_binary
            (i27::MAX as u32 + 1, vec![0x62, 0x08, 0x00, 0x00, 0x00]),
            (i32::MAX as u32,     vec![0x62, 0x7F, 0xFF, 0xFF, 0xFF]),
        ] {
            let actual: Result<u32, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_u64() {
        for (expected, input) in vec![
            // ei
            ( u8::MIN as u64,     vec![0x61, 0x00]),
            ( u8::MIN as u64 + 1, vec![0x61, 0x01]),
            ( u8::MAX as u64,     vec![0x61, 0xff]),
            ( u8::MAX as u64 + 1, vec![0x62, 0x00, 0x00, 0x01, 0x00]),
            (i27::MAX as u64,     vec![0x62, 0x07, 0xff, 0xff, 0xff]),
            (i27::MAX as u64 + 1, vec![0x6e, 0x04, 0x00, 0x00, 0x00, 0x00, 0x08]),
            (u64::MAX,            vec![0x6e, 0x08, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            // term_to_binary
            (i27::MAX as u64 + 1, vec![0x62, 0x08, 0x00, 0x00, 0x00]),
            (i32::MAX as u64,     vec![0x62, 0x7F, 0xFF, 0xFF, 0xFF]),
            (i32::MAX as u64 + 1, vec![0x6E, 0x04, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ] {
            let actual: Result<u64, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_f64() {
        for (expected, input) in vec![
            (-1.0 as f64, vec![0x46, 0xbf, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (-0.0 as f64, vec![0x46, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            ( 0.0 as f64, vec![0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            ( 1.0 as f64, vec![0x46, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ] {
            let actual: Result<f64, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_string() {
        for (expected, input) in vec![
            ("",      vec![0x6a]),
            ("hello", vec![0x6b, 0x00, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        ] {
            let actual: Result<String, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_bytes() {
        for (expected, input) in vec![
            ("",      vec![0x6d, 0x00, 0x00, 0x00, 0x00]),
            ("hello", vec![0x6d, 0x00, 0x00, 0x00, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        ] {
            let actual: Result<CString, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(CString::new(expected).unwrap(), actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_unit_struct() {
        let input = vec![0x6a];
        let actual: Result<Instance, Error> = test!(&input);
        assert!(actual.is_ok(), "{:?}", actual);
    }

    #[test]
    fn deserialize_newtype_struct() {
        let input = vec![0x68, 0x01, 0x6b, 0x00, 0x01, 0x61];
        let actual: Result<Inches, Error> = test!(&input);
        assert!(actual.is_ok(), "{:?}", actual);
        assert_eq!(Inches("a".to_owned()), actual.unwrap(), "{:?}", input);
    }

    #[test]
    fn deserialize_seq() {
        for (expected, input) in vec![
            (vec![],  vec![0x6a]),
            (vec![1], vec![0x6c, 0x00, 0x00, 0x00, 0x01, 0x61, 0x01, 0x6a]),
        ] {
            let actual: Result<Vec<i32>, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_tuple() {
        {
            let input = vec![0x6a];
            let actual: Result<(), Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
        }
        {
            let input = vec![0x68, 0x01, 0x61, 0x01];
            let actual: Result<(u8,), Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!((1,), actual.unwrap(), "{:?}", input);
        }
        {
            let input = vec![0x68, 0x02, 0x61, 0x01, 0x77, 0x04, 0x74, 0x72, 0x75, 0x65];
            let actual: Result<(u8,bool), Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!((1,true), actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_tuple_struct() {
        let input = vec![0x68, 0x02, 0x61, 0x01, 0x61, 0x02];
        let actual: Result<Point2D, Error> = test!(&input);
        assert!(actual.is_ok(), "{:?}", actual);
        assert_eq!(Point2D(1, 2), actual.unwrap(), "{:?}", input);
    }

    #[test]
    fn deserialize_map() {
        let mut map = HashMap::new();

        {
            map.clear();

            let input = vec![0x74, 0x00, 0x00, 0x00, 0x00];
            let actual: Result<HashMap<u8,u8>, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(map, actual.unwrap(), "{:?}", input);
        }
        {
            map.clear();
            map.insert(1, 2);

            let input = vec![0x74, 0x00, 0x00, 0x00, 0x01, 0x61, 0x01, 0x61, 0x02];
            let actual: Result<HashMap<u8,u8>, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(map, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_struct() {
        let input = vec![
            0x68, 0x03,
            0x61, 0x01,
            0x61, 0x02,
            0x46, 0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        let actual: Result<Color, Error> = test!(&input);
        assert!(actual.is_ok(), "{:?}", actual);
        assert_eq!(Color { r: 1, g: 2, b: 3 as f64}, actual.unwrap(), "{:?}", input);
    }

    #[test]
    fn deserialize_atom() {
        for (expected, input) in vec![
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
            let actual: Result<term::Atom, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_pid() {
        for (expected, input) in vec![
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
            let actual: Result<term::Pid, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_port() {
        for (expected, input) in vec![
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
            let actual: Result<term::Port, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_ref() {
        for (expected, input) in vec![
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
            let actual: Result<term::Ref, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_trace() {
        for (expected, input) in vec![
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
            let actual: Result<term::Trace, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }

    #[test]
    fn deserialize_msg() {
        for (expected, input) in vec![
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
            let actual: Result<term::Msg, Error> = test!(&input);
            assert!(actual.is_ok(), "{:?}", actual);
            assert_eq!(expected, actual.unwrap(), "{:?}", input);
        }
    }
}
