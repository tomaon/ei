use std::net;
use std::thread;

extern crate ei;

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

                                let value: u64 = try!(ei::decode(r));

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
