use std::net;
use std::thread;

extern crate ei;

// @see
//  http://qiita.com/tatsuya6502/items/52c2817b5ccae6d51197

const MAX_THREADS: u32 = 64;

#[allow(dead_code)]
pub fn calc_pi_parallel(n: u32, num_threads: u32) -> Result<f64, String> {
    if num_threads <= 0 || num_threads > MAX_THREADS {
        Err(format!("Invalid num_threads {}. It must be > 0 and <= {}",
                    num_threads, MAX_THREADS))
    } else if n % num_threads != 0 {
        Err(format!("n {} must be a multiple of num_threads {}",
                    n, num_threads))
    } else {
        let len = n / num_threads;
        let handles: Vec<_> = (0..num_threads).map(|i| {
            thread::spawn(move || {
                calc_pi_range(n, len * i, len)
            })
        }).collect();

        let results = handles.into_iter().map(|h| { h.join().unwrap() });
        // std::iter::Iterator の sum() は Rust 1.5 では unstable に
        // 指定されており使えない。代わりに fold() を使う。
        let pi: f64 = results.into_iter().fold(0.0, |acc, p| { acc + p });
        Ok(pi)
    }
}

fn calc_pi_range(n: u32, offset: u32, count: u32) -> f64 {
    let w = 1.0 / (n as f64);
    let mut s = 0.0;
    for i in offset..(offset + count) {
        let x = (i as f64) * w;
        s += (1.0 - x * x).sqrt();
    }
    4.0 * w * s
}

fn main() {

    let node = "r1@localhost";
    let port = 3456;

    let v: Vec<&str> = node.split('@').collect();

    match net::TcpListener::bind((v[1], port)) {
        Ok(listener) => {

            println!("local: {:?}", listener.local_addr().unwrap());

            if let Ok(epmd) = ei::publish(v[0], port) {

                println!("epmd : {:?}", ei::port(v[0]).unwrap());

                for item in listener.incoming() {
                    match item {
                        Ok(mut stream) => {

                            println!("peer : {:?}", stream.peer_addr().unwrap());

                            thread::spawn(move || ei::handle(&mut stream, &node.to_string(), |r| {

                                let (n, num_threads): (u32, u32) = try!(ei::decode(r));

                                let value = calc_pi_parallel(n, num_threads).unwrap();

                                let result = (ei::Atom::from("ok"), value);

                                let mut buf = Vec::new();
                                ei::encode(&mut buf, &result).and(Ok(buf))

                            }));
                        },
                        Err(e) => println!("{:?}", e),
                    }
                }

                drop(epmd);
            }

            drop(listener);
        },
        Err(e) => println!("{:?}", e),
    }
}
