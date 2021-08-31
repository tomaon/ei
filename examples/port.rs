use std::io;
use std::sync::mpsc;
use std::thread;

use ei;

fn main() {
    let (sender, receiver) = mpsc::channel::<Vec<u8>>();

    // port -> beam
    let mut writer = ei::Writer::new(io::stdout());

    thread::spawn(move || {
        while let Ok(vec) = receiver.recv() {
            writer.send(&vec).unwrap()
        }
    });

    // beam -> port
    let mut reader = ei::Reader::new(io::stdin());

    while let Ok(vec) = reader.recv() {
        let s = sender.clone();
        thread::spawn(move || {
            let req: (u8, i64, i64, ei::Pid, ei::Atom, ei::Ref) =
                ei::deserialize!(vec.as_slice()).unwrap();

            let res = match req.0 {
                0x61 => {
                    let res = (ei::atom!("ok"), add(req.1, req.2), req.3, req.4, req.5);
                    ei::serialize!(&res).unwrap()
                }
                0x73 => {
                    let res = (ei::atom!("ok"), sub(req.1, req.2), req.3, req.4, req.5);
                    ei::serialize!(&res).unwrap()
                }
                _ => {
                    let res = (ei::atom!("error"), ei::atom!("undef"), req.3, req.4, req.5);
                    ei::serialize!(&res).unwrap()
                }
            };

            s.send(res).unwrap();
        });
    }
}

fn add(v1: i64, v2: i64) -> i64 {
    v1 + v2
}

fn sub(v1: i64, v2: i64) -> i64 {
    v1 - v2
}
