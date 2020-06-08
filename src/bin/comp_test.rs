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
    let (offset_buf, rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
    println!("{:?}", TTFOffsetTable::decode_from_be_bytes(offset_buf));

    let (header, rest) = rest.split_at(TTFTableHeader::PACKED_LEN);
    let table_header = TTFTableHeader::decode_from_be_bytes(header);
    let head = head::Head::decode_from_be_bytes(rest);
    println!("{:?}\n\n{:?}\n", table_header, head);

    println!("\n{:?}", buf);
    Ok(())
}

fn checksum_any<T: PackedSize + EncodeBE>(p: &T) -> u32 {
    let mut buf = vec![0u8; T::PACKED_LEN];
    p.encode_as_be_bytes(&mut buf[..]);
    return checksum(&buf);
}

const fn align_len(len: usize) -> usize {
    let round_up = (4usize - (len & 0x3)) & 0x3;
    return len + round_up;
}

fn table_len<T: PackedSize>(_: &T) -> usize {
    return align_len(T::PACKED_LEN);
}

fn header_for<T: PackedSize + EncodeBE>(tag: u32,
        offset_from_start_of_file: usize, p: &T) -> TTFTableHeader {
    TTFTableHeader {
        tag,
        checksum: checksum_any(p),
        offset_from_start_of_file: align_len(offset_from_start_of_file
            + TTFTableHeader::PACKED_LEN) as u32,
        length: T::PACKED_LEN as u32
    }
}

fn write_into<T: PackedSize + EncodeBE>(v: &mut Vec<u8>, p: &T) {
    let start = v.len();
    v.resize(start + table_len(p), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

const fn tag_const(x: &[u8; 4]) -> u32 {
    return (x[0] as u32) << 24
         | (x[1] as u32) << 16
         | (x[2] as u32) << 8
         | (x[3] as u32);
}

fn write_ttf(_path: &str) -> io::Result<()> {
    let offset_table = TTFOffsetTable {
        version: 0x00010000,
        num_tables: 1,
        search_range: 16,
        entry_selector: 0,
        range_shift: 0
    };

    let mut head = head::Head::new();

    // all stuff to get a clean diff between our output and `spec9c1.ttf`
    head.magic_number = 0;
    head.font_revision = head::Fixed1616::from_bits(72090);
    head.created = 3406620153;
    head.modified = 3647951938;
    head.font_direction_hint = 0;

    let head_header = header_for(
        tag_const(b"head"),
        TTFOffsetTable::PACKED_LEN,
        &head);

    println!("{} {:?}", TTFOffsetTable::PACKED_LEN, head_header);

    head.checksum_adjustment = 5023306;

    let mut buf = Vec::new();

    write_into(&mut buf, &offset_table);
    write_into(&mut buf, &head_header);

    head.checksum_adjustment = 0xB1B0AFBA -
        checksum(&buf).overflowing_add(head_header.checksum).0;

    write_into(&mut buf, &head);

    println!("{:?}", buf);
    println!("{:?}", head);

    let mut f = File::create("ours.ttf")?;
    f.write(&buf)?;

    Ok(())
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    read_header(&path).unwrap();
    println!("\n---\n");
    write_ttf(&path).unwrap();
}
