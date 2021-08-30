#[macro_export]
macro_rules! atom {
    ($e: expr) => {
        $crate::Atom::from($e)
    };
}

#[macro_export]
macro_rules! deserialize {
    ($e: expr) => {
        $crate::from_reader($e)
    };
}

#[macro_export]
macro_rules! serialize {
    ($v: expr) => {
        $crate::to_vec($v, 256)
    };
    ($v: expr, $u: expr) => {
        $crate::to_vec($v, $u)
    };
}

macro_rules! range {
    ($v: expr, $t: tt, $c:ty) => {
        $v as $c >= $t::MIN as $c && $v as $c <= $t::MAX as $c
    };
    ($v: expr, $f: expr, $t: expr, $c:ty) => {
        $v as $c >= $f as $c && $v as $c <= $t as $c
    };
}

macro_rules! interrupted {
    ($($arg:tt)+) => ($crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Interrupted, format!("{}", format_args!($($arg)+)))));
}

macro_rules! invalid_data {
    ($($arg:tt)+) => ($crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", format_args!($($arg)+)))));
}

macro_rules! invalid_input {
    ($($arg:tt)+) => ($crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{}", format_args!($($arg)+)))));
}

// macro_rules! not_found {
//     ($($arg:tt)+) => ($crate::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}", format_args!($($arg)+)))));
// }

macro_rules! unsupported {
    ($($arg:tt)+) => ($crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{}", format_args!($($arg)+)))));
}
