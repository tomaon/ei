use std::env;
use std::fs;
use std::io::Read;

pub fn cookie() -> String {
    if let Some(mut path) = env::home_dir() {
        path.push(".erlang.cookie");
        if let Ok(mut f) = fs::File::open(&path) {
            let mut buf = String::new();
            if let Ok(_) = f.read_to_string(&mut buf) {
                return buf;
            }
        }
    }
    "".to_string()
}
