use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;

extern crate ei;

mod calc_pi;

macro_rules! atom {
    ($e: expr) => (ei::Atom::from($e));
}

fn main() {

    let (sender, receiver) = mpsc::channel::<Vec<u8>>();

    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(vec) => if let Err(e) = ei::send(&mut io::stdout(), vec.as_slice()) {
                    panic!("{:?}", e);
                },
                Err(_) => break,
            }
        }
    });

    loop {
        match ei::recv(&mut io::stdin()) {
            Ok(mut cursor) => {
                let s = sender.clone();
                thread::spawn(move || {

                    let req: (u64,u64,(ei::Pid,ei::Ref)) = ei::decode(&mut cursor).unwrap();

                    let mut buf = Vec::new();

                    match req.0 {
                        0x66 => ei::encode(&mut buf, &(atom!("ok"), foo(req.1), req.2)).unwrap(),
                        0x62 => ei::encode(&mut buf, &(atom!("ok"), bar(req.1), req.2)).unwrap(),
                        0x7a => ei::encode(&mut buf, &(atom!("error"), atom!("badarg"), req.2)).unwrap(),
                        _    => ei::encode(&mut buf, &(atom!("ok"), pi(req.0,req.1), req.2)).unwrap(),
                    }

                    if let Err(e) = s.send(buf) {
                        panic!("{:?}", e);
                    }
                });
            },
            Err(e) => match e.description() { // TODO
                "operation interrupted" => break,
                _                       => panic!("{:?}", e),
            },
        }
    }
}

fn foo(val: u64) -> u64 {
    val + 1
}

fn bar(val: u64) -> u64 {
    val * 2
}

fn pi(n: u64, num_threads: u64) -> f64 {
    match calc_pi::calc_pi_parallel(n as u32, num_threads as u32) {
        Ok(f)  => f,
        Err(_) => unimplemented!(),
    }
}
