use std::io::{Read, Write};
use std::mem;
use std::ptr;
use std::result;

use convert::AsMutPtr;
use error;

macro_rules! copy_nonoverlapping {
    ($src: expr, $dst: ty, $count: expr) => ({
        let mut e: $dst = 0;
        unsafe {
            ptr::copy_nonoverlapping($src.as_ptr(), e.as_mut_ptr(), $count);
        }
        e.to_be()
    });
}

macro_rules! transmute {
    ($e: expr, $src: ty, $size: expr) => (
        unsafe {
            mem::transmute::<$src, [u8; $size]>($e.to_be()).as_ref()
        }
    );
}

pub type Result<T> = result::Result<T, error::Error>;

pub trait ReadExt {
    fn read_u64(&mut self) -> Result<u64>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_i8(&mut self) -> Result<i8>;
    fn read_f64(&mut self) -> Result<f64>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_string(&mut self, size: usize) -> Result<String>;
    fn read_vec(&mut self, size: usize) -> Result<Vec<u8>>;
}

impl<T: Read + ?Sized> ReadExt for T {

    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        match self.read_exact(&mut buf) {
            Ok(()) => Ok(copy_nonoverlapping!(buf, u64, 8)),
            Err(e) => Err(From::from(e))
        }
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        match self.read_exact(&mut buf) {
            Ok(()) => Ok(copy_nonoverlapping!(buf, u32, 4)),
            Err(e) => Err(From::from(e)),
        }
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        match self.read_exact(&mut buf) {
            Ok(()) => Ok(copy_nonoverlapping!(buf, u16, 2)),
            Err(e) => Err(From::from(e)),
        }
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        match self.read_exact(&mut buf) {
            Ok(()) => Ok(buf[0]),
            Err(e) => Err(From::from(e)),
        }
    }

    fn read_i64(&mut self) -> Result<i64> {
        self.read_u64().and_then(|u| Ok(u as i64))
    }

    fn read_i32(&mut self) -> Result<i32> {
        self.read_u32().and_then(|u| Ok(u as i32))
    }

    fn read_i16(&mut self) -> Result<i16> {
        self.read_u16().and_then(|u| Ok(u as i16))
    }

    fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().and_then(|u| Ok(u as i8))
    }

    fn read_f64(&mut self) -> Result<f64> {
        self.read_u64().and_then(|u| Ok(unsafe { mem::transmute(u) }))
    }

    fn read_f32(&mut self) -> Result<f32> {
        self.read_u32().and_then(|u| Ok(unsafe { mem::transmute(u) }))
    }

    fn read_string(&mut self, size: usize) -> Result<String> {
        self.read_vec(size).and_then(|v| String::from_utf8(v).or_else(|e| Err(From::from(e))))
    }

    fn read_vec(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size];
        match self.read_exact(&mut buf) {
            Ok(()) => Ok(buf),
            Err(e) => Err(From::from(e)),
        }
    }
}

pub trait WriteExt {
    fn write_u64(&mut self, v: u64) -> Result<()>;
    fn write_u32(&mut self, v: u32) -> Result<()>;
    fn write_u16(&mut self, v: u16) -> Result<()>;
    fn write_u8(&mut self, v: u8) -> Result<()>;
    fn write_i64(&mut self, v: i64) -> Result<()>;
    fn write_i32(&mut self, v: i32) -> Result<()>;
    fn write_i16(&mut self, v: i16) -> Result<()>;
    fn write_i8(&mut self, v: i8) -> Result<()>;
    fn write_f64(&mut self, v: f64) -> Result<()>;
    fn write_f32(&mut self, v: f32) -> Result<()>;
    fn write_slice(&mut self, v: &[u8]) -> Result<()>;
}

impl<T: Write + ?Sized> WriteExt for T {

    fn write_u64(&mut self, v: u64) -> Result<()> {
        self.write_slice(transmute!(v, u64, 8))
    }

    fn write_u32(&mut self, v: u32) -> Result<()> {
        self.write_slice(transmute!(v, u32, 4))
    }

    fn write_u16(&mut self, v: u16) -> Result<()> {
        self.write_slice(transmute!(v, u16, 2))
    }

    fn write_u8(&mut self, v: u8) -> Result<()> {
        self.write_slice(&[v])
    }

    fn write_i64(&mut self, v: i64) -> Result<()> {
        self.write_slice(transmute!(v, i64, 8))
    }

    fn write_i32(&mut self, v: i32) -> Result<()> {
        self.write_slice(transmute!(v, i32, 4))
    }

    fn write_i16(&mut self, v: i16) -> Result<()> {
        self.write_slice(transmute!(v, i16, 2))
    }

    fn write_i8(&mut self, v: i8) -> Result<()> {
        self.write_slice(&[v as u8])
    }

    fn write_f64(&mut self, v: f64) -> Result<()> {
        self.write_u64(unsafe { mem::transmute(v) })
    }

    fn write_f32(&mut self, v: f32) -> Result<()> {
        self.write_u32(unsafe { mem::transmute(v) })
    }

    fn write_slice(&mut self, v: &[u8]) -> Result<()> {
        self.write_all(v).or_else(|e| Err(From::from(e)))
    }
}

#[cfg(test)]
mod tests {

    mod read {

        use super::super::ReadExt;

        macro_rules! test {
            ($i: expr, $f: ident) => ($i.as_slice().$f());
        }

        #[test]
        fn read_u64() {
            for (expected, input) in vec![
                (                   0 as u64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                   1 as u64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01]),
                (18446744073709551615 as u64, vec![0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_u64).unwrap());
            }
        }

        #[test]
        fn read_u32() {
            for (expected, input) in vec![
                (         0 as u32, vec![0x00,0x00,0x00,0x00]),
                (         1 as u32, vec![0x00,0x00,0x00,0x01]),
                (4294967295 as u32, vec![0xff,0xff,0xff,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_u32).unwrap());
            }
        }

        #[test]
        fn read_u16() {
            for (expected, input) in vec![
                (    0 as u16, vec![0x00,0x00]),
                (    1 as u16, vec![0x00,0x01]),
                (65535 as u16, vec![0xff,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_u16).unwrap());
            }
        }

        #[test]
        fn read_u8() {
            for (expected, input) in vec![
                (  0 as u8, vec![0x00]),
                (  1 as u8, vec![0x01]),
                (255 as u8, vec![0xff]),
            ] {
                assert_eq!(expected, test!(input, read_u8).unwrap());
            }
        }

        #[test]
        fn read_i64() {
            for (expected, input) in vec![
                (-9223372036854775808 as i64, vec![0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                  -1 as i64, vec![0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
                (                   0 as i64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                   1 as i64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01]),
                ( 9223372036854775807 as i64, vec![0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_i64).unwrap());
            }
        }

        #[test]
        fn read_i32() {
            for (expected, input) in vec![
                (-2147483648 as i32, vec![0x80,0x00,0x00,0x00]),
                (         -1 as i32, vec![0xff,0xff,0xff,0xff]),
                (          0 as i32, vec![0x00,0x00,0x00,0x00]),
                (          1 as i32, vec![0x00,0x00,0x00,0x01]),
                ( 2147483647 as i32, vec![0x7f,0xff,0xff,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_i32).unwrap());
            }
        }

        #[test]
        fn read_i16() {
            for (expected, input) in vec![
                (-32768 as i16, vec![0x80,0x00]),
                (    -1 as i16, vec![0xff,0xff]),
                (     0 as i16, vec![0x00,0x00]),
                (     1 as i16, vec![0x00,0x01]),
                ( 32767 as i16, vec![0x7f,0xff]),
            ] {
                assert_eq!(expected, test!(input, read_i16).unwrap());
            }
        }

        #[test]
        fn read_i8() {
            for (expected, input) in vec![
                (-128 as i8, vec![0x80]),
                (  -1 as i8, vec![0xff]),
                (   0 as i8, vec![0x00]),
                (   1 as i8, vec![0x01]),
                ( 127 as i8, vec![0x7f]),
            ] {
                assert_eq!(expected, test!(input, read_i8).unwrap());
            }
        }

        #[test]
        fn read_f64() {
            for (expected, input) in vec![
                (-1.0 as f64, vec![0xbf,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
                (-0.0 as f64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                ( 0.0 as f64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                ( 1.0 as f64, vec![0x3f,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
            ] {
                assert_eq!(expected, test!(input, read_f64).unwrap());
            }
        }

        #[test]
        fn read_f32() {
            for (expected, input) in vec![
                (-1.0 as f32, vec![0xbf,0x80,0x00,0x00]),
                (-0.0 as f32, vec![0x80,0x00,0x00,0x00]),
                ( 0.0 as f32, vec![0x00,0x00,0x00,0x00]),
                ( 1.0 as f32, vec![0x3f,0x80,0x00,0x00]),
            ] {
                assert_eq!(expected, test!(input, read_f32).unwrap());
            }
        }
    }

    mod write {

        use super::super::WriteExt;

        macro_rules! test {
            ($o: expr, $i: expr, $f: ident) => ($o.$f($i).and(Ok($o.as_slice())));
        }

        #[test]
        fn write_u64() {
            for (input, expected) in vec![
                (                   0 as u64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                   1 as u64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01]),
                (18446744073709551615 as u64, vec![0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_u64).unwrap());
            }
        }

        #[test]
        fn write_u32() {
            for (input, expected) in vec![
                (         0 as u32, vec![0x00,0x00,0x00,0x00]),
                (         1 as u32, vec![0x00,0x00,0x00,0x01]),
                (4294967295 as u32, vec![0xff,0xff,0xff,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_u32).unwrap());
            }
        }

        #[test]
        fn write_u16() {
            for (input, expected) in vec![
                (    0 as u16, vec![0x00,0x00]),
                (    1 as u16, vec![0x00,0x01]),
                (65535 as u16, vec![0xff,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_u16).unwrap());
            }
        }

        #[test]
        fn write_u8() {
            for (input, expected) in vec![
                (  0 as u8, vec![0x00]),
                (  1 as u8, vec![0x01]),
                (255 as u8, vec![0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_u8).unwrap());
            }
        }

        #[test]
        fn write_i64() {
            for (input, expected) in vec![
                (-9223372036854775808 as i64, vec![0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                  -1 as i64, vec![0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
                (                   0 as i64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                (                   1 as i64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01]),
                ( 9223372036854775807 as i64, vec![0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_i64).unwrap());
            }
        }

        #[test]
        fn write_i32() {
            for (input, expected) in vec![
                (-2147483648 as i32, vec![0x80,0x00,0x00,0x00]),
                (         -1 as i32, vec![0xff,0xff,0xff,0xff]),
                (          0 as i32, vec![0x00,0x00,0x00,0x00]),
                (          1 as i32, vec![0x00,0x00,0x00,0x01]),
                ( 2147483647 as i32, vec![0x7f,0xff,0xff,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_i32).unwrap());
            }
        }

        #[test]
        fn write_i16() {
            for (input, expected) in vec![
                (-32768 as i16, vec![0x80,0x00]),
                (    -1 as i16, vec![0xff,0xff]),
                (     0 as i16, vec![0x00,0x00]),
                (     1 as i16, vec![0x00,0x01]),
                ( 32767 as i16, vec![0x7f,0xff]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_i16).unwrap());
            }
        }

        #[test]
        fn write_i8() {
            for (input, expected) in vec![
                (-128 as i8, vec![0x80]),
                (  -1 as i8, vec![0xff]),
                (   0 as i8, vec![0x00]),
                (   1 as i8, vec![0x01]),
                ( 127 as i8, vec![0x7f]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_i8).unwrap());
            }
        }

        #[test]
        fn write_f64() {
            for (input, expected) in vec![
                (-1.0 as f64, vec![0xbf,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
                (-0.0 as f64, vec![0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                ( 0.0 as f64, vec![0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]),
                ( 1.0 as f64, vec![0x3f,0xf0,0x00,0x00,0x00,0x00,0x00,0x00]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_f64).unwrap());
            }
        }

        #[test]
        fn write_f32() {
            for (input, expected) in vec![
                (-1.0 as f32, vec![0xbf,0x80,0x00,0x00]),
                (-0.0 as f32, vec![0x80,0x00,0x00,0x00]),
                ( 0.0 as f32, vec![0x00,0x00,0x00,0x00]),
                ( 1.0 as f32, vec![0x3f,0x80,0x00,0x00]),
            ] {
                let mut output: Vec<u8> = Vec::new();
                assert_eq!(expected, test!(&mut output, input, write_f32).unwrap());
            }
        }
    }
}
