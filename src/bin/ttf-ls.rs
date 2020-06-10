use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use endian_codec::{PackedSize, DecodeBE};

use otf_fea_rs::compile_model::*;

fn read_header(path: &str) -> io::Result<()> {
    let mut f = File::open(path)?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let (offset_buf, mut rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
    let offset_table = TTFOffsetTable::decode_from_be_bytes(offset_buf);
    println!("{:#?}\n", offset_table);

    for _ in 0..offset_table.num_tables {
        let (header_buf, r) = rest.split_at(TTFTableHeader::PACKED_LEN);
        let header = TTFTableHeader::decode_from_be_bytes(header_buf);
        println!("{:#?}", header);
        rest = r;
    }

    Ok(())
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    read_header(&path).unwrap();
}
