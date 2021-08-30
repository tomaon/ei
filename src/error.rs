use std::error;
use std::fmt;
use std::io;
use std::str;
use std::string;

use serde;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Str(str::Utf8Error),
    String(string::FromUtf8Error),
    Custom(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(ref e) => e.fmt(f),
            Error::Str(ref e) => e.fmt(f),
            Error::String(ref e) => e.fmt(f),
            Error::Custom(_) => unimplemented!("fmt"),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        unimplemented!("custom")
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        unimplemented!("custom")
    }
}
