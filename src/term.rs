#[derive(Debug, PartialEq)]
pub enum Atom {
    Latin1(String),
    UTF8(String),
    UTF8Small(String),
}

impl<'a> From<&'a str> for Atom {
    fn from(s: &'a str) -> Self {
        Atom::Latin1(s.to_string())
    }
}

// TODO: big | mpz_t (GNU MP)

// TODO: binary

// TODO: fun

#[derive(Debug, PartialEq)]
pub enum Msg {
    Send   { cookie: Atom, to: Pid },
    SendTT { cookie: Atom, to: Pid , token: Trace },
    RegSend   { from: Pid, cookie: Atom, toname: Atom },
    RegSendTT { from: Pid, cookie: Atom, toname: Atom, token: Trace },
    Exit   { from: Pid, to: Pid, reason: String },
    ExitTT { from: Pid, to: Pid, token: Trace, reason: String },
}

#[derive(Debug, PartialEq)]
pub struct Pid {
    pub node: Atom,
    pub num: u32,
    pub serial: u32,
    pub creation: u32,
}

#[derive(Debug, PartialEq)]
pub struct Port {
    pub node: Atom,
    pub id: u32,
    pub creation: u32,
}

#[derive(Debug, PartialEq)]
pub struct Ref {
    pub len: i16,
    pub node: Atom,
    pub creation: u32,
    pub n: [u32; 3]
}

#[derive(Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Trace(
    pub i64, // 0: flags
    pub i64, // 1: label
    pub i64, // 2: serial
    pub Pid, // 3: Pid
    pub i64, // 4: prev
);
