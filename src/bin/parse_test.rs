use std::env;
use std::fs::File;

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    let f = File::open(&path).unwrap();

    match otf_fea_rs::parser::parse_all(f) {
        Ok(definitions) => {
            if env::var("FEA_RS_NO_PRINT").is_err() {
                println!();

                for d in definitions {
                    println!("{:#?}\n", d);
                }
            }

            std::process::exit(0)
        },

        Err(_) => std::process::exit(1)
    }
}
