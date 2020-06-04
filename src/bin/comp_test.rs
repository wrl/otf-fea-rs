#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use otf_fea_rs::compile_model::*;

fn checksum_head(table: &[u8]) -> u32 {
    return table.chunks(4)
        .enumerate()
        .fold(0u32, |acc, (i, bytes)| {
            let raw = match (i, bytes) {
                // for the `head` table, we have to treat `checksum_adjustment` as 0 while calculating
                // the checksum for the header.
                (2, _) => [0, 0, 0, 0],

                (_, &[a]) => [a, 0, 0, 0],
                (_, &[a, b]) => [a, b, 0, 0],
                (_, &[a, b, c]) => [a, b, c, 0],
                (_, &[a, b, c, d]) => [a, b, c, d],
                _ => unreachable!()
            };

            return acc.overflowing_add(u32::from_be_bytes(raw)).0;
        });
}

fn checksum(table: &[u8]) -> u32 {
    return table.chunks(4)
        .fold(0u32, |acc, bytes| {
            let raw = match bytes {
                &[a] => [a, 0, 0, 0],
                &[a, b] => [a, b, 0, 0],
                &[a, b, c] => [a, b, c, 0],
                &[a, b, c, d] => [a, b, c, d],
                _ => unreachable!()
            };

            return acc.overflowing_add(u32::from_be_bytes(raw)).0;
        });
}

fn read_header(path: &str) -> io::Result<()> {
    let mut f = File::open(path)?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    println!();
    let (hdr, rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
    println!("{:?}", TTFOffsetTable::decode_from_be_bytes(hdr));

    let table_header = TTFTableHeader::decode_from_be_bytes(rest);
    let head = head::Head::decode_from_be_bytes(rest);
    println!("{:?}\n\n{:?}\n", table_header, head);

    let head_bytes = rest.split_at(TTFTableHeader::PACKED_LEN).1;
    println!("{}", checksum_head(&head_bytes[..(table_header.length as usize)]));

    Ok(())
}

fn write_header(path: &str, hdr: &TTFOffsetTable) {
    let mut buf = [0u8; TTFOffsetTable::PACKED_LEN];

    hdr.encode_as_be_bytes(&mut buf);

    let mut f = File::create(path).unwrap();
    f.write(&buf).unwrap();
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    read_header(&path).unwrap();
    // write_header(&path, &SFNTHeader {
    //     version: 0x00010000,
    //     num_tables: 0,
    //     search_range: 16,
    //     entry_selector: 0,
    //     range_shift: 0
    // });
}
