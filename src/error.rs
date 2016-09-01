use std::error;
use std::fmt;
use std::io;
use std::string;

macro_rules! from_raw_os_error {
    ($e: expr) => (error::Error::from_raw_os_error($e));
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FromUtf8(string::FromUtf8Error),
}

impl Error {

    pub fn from_raw_os_error(code: i32) -> Error {
        From::from(io::Error::from_raw_os_error(code))
    }
}

impl error::Error for Error {

    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::FromUtf8(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => e.cause(),
            Error::FromUtf8(ref e) => e.cause(),
        }
    }
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::FromUtf8(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {

    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<string::FromUtf8Error> for Error {

    fn from(e: string::FromUtf8Error) -> Error {
        Error::FromUtf8(e)
    }
}
