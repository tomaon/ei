use serde::{Deserialize, Serialize};

#[derive(Serialize, PartialEq, Debug)]
pub enum Atom {
    Latin1(String),
    UTF8Small(String),
    UTF8(String),
}

impl<'a> From<&'a str> for Atom {
    fn from(s: &'a str) -> Self {
        Atom::UTF8Small(s.to_string())
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Pid {
    pub node: Atom,
    pub num: u32,
    pub serial: u32,
    pub creation: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum Port {
    NewPort { node: Atom, id: u32, creation: u32 },
    V4Port { node: Atom, id: u64, creation: u32 },
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Ref {
    pub len: i16, // 0..5
    pub node: Atom,
    pub creation: u32,
    pub n: Option<Vec<u32>>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Trace(
    pub i64, // 0: flags
    pub i64, // 1: label
    pub i64, // 2: serial
    pub Pid, // 3: from
    pub i64, // 4: prev
);

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum Msg {
    Send {
        cookie: Atom,
        to: Pid,
    },
    SendTT {
        cookie: Atom,
        to: Pid,
        token: Trace,
    },
    RegSend {
        from: Pid,
        cookie: Atom,
        toname: Atom,
    },
    RegSendTT {
        from: Pid,
        cookie: Atom,
        toname: Atom,
        token: Trace,
    },
    Exit {
        from: Pid,
        to: Pid,
        reason: String,
    },
    ExitTT {
        from: Pid,
        to: Pid,
        token: Trace,
        reason: String,
    },
}
