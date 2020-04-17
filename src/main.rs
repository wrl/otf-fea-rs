use std::env;
use std::fs::File;

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    let f = File::open(&path).unwrap();

    match otf_fea_rs::parse(f) {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(1)
    }
}
