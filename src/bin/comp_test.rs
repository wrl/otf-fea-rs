#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::prelude::*;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use otf_fea_rs::compile_model::*;

fn read_header(path: &str) {
    let mut f = File::open(path).unwrap();

    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    println!("{:?}", SFNTHeader::decode_from_be_bytes(&buf));
}

fn write_header(path: &str, hdr: &SFNTHeader) {
    let mut buf = [0u8; SFNTHeader::PACKED_LEN];

    hdr.encode_as_be_bytes(&mut buf);

    let mut f = File::create(path).unwrap();
    f.write(&buf).unwrap();
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    // read_header(&path);
    write_header(&path, &SFNTHeader {
        version: 0x00010000,
        num_tables: 0,
        search_range: 16,
        entry_selector: 0,
        range_shift: 0
    });
}
