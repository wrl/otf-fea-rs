#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use otf_fea_rs::compile_model::*;
use otf_fea_rs::Tag;

fn read_header(path: &str) -> io::Result<()> {
    let mut f = File::open(path)?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    println!();
    let (offset_buf, rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
    println!("{:?}", TTFOffsetTable::decode_from_be_bytes(offset_buf));

    let (record, rest) = rest.split_at(TTFTableRecord::PACKED_LEN);
    let table_record = TTFTableRecord::decode_from_be_bytes(record);
    let head = tables::Head::decode_from_be_bytes(rest);
    println!("{:?}\n\n{:#?}\n", table_record, head);

    println!(" >> created: {:?}", head.created.as_datetime());
    println!(" >> modified: {:?}", head.modified.as_datetime());
    println!(" >> revision: {}", head.font_revision.to_bits());

    println!("\n{:?}", buf);
    Ok(())
}

fn checksum_any<T: PackedSize + EncodeBE>(p: &T) -> u32 {
    let mut buf = vec![0u8; T::PACKED_LEN];
    p.encode_as_be_bytes(&mut buf[..]);

    // don't need to handle the checksum_head() special case here because, at this phase in
    // compilation, the `checksum_adjustment` field is 0 anyway.
    return util::checksum(&buf);
}

const fn align_len(len: usize) -> usize {
    let round_up = (4usize - (len & 0x3)) & 0x3;
    return len + round_up;
}

fn table_len<T: PackedSize>(_: &T) -> usize {
    return align_len(T::PACKED_LEN);
}

fn record_for<T: PackedSize + EncodeBE>(tag: Tag,
    offset_from_start_of_file: usize, p: &T) -> TTFTableRecord {
    TTFTableRecord {
        tag,
        checksum: checksum_any(p),
        offset_from_start_of_file: align_len(offset_from_start_of_file
            + TTFTableRecord::PACKED_LEN) as u32,
        length: T::PACKED_LEN as u32
    }
}

fn write_into<T: PackedSize + EncodeBE>(v: &mut Vec<u8>, p: &T) {
    let start = v.len();
    v.resize(start + table_len(p), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

fn write_ttf(_path: &str) -> io::Result<()> {
    let offset_table = TTFOffsetTable {
        version: TTFVersion::TTF,
        num_tables: 1,
        search_range: 16,
        entry_selector: 0,
        range_shift: 0
    };

    let mut head = tables::Head::new();

    // all stuff to get a clean diff between our output and `spec9c1.ttf`
    head.magic_number = 0;
    head.font_revision = util::Fixed1616::from_bits(72090);
    head.created = 3406620153.into();
    head.modified = 3647951938.into();
    head.font_direction_hint = 0;

    let head_record = record_for(
        Tag::from_bytes(b"head").unwrap(),
        TTFOffsetTable::PACKED_LEN,
        &head);

    println!("{} {:?}", TTFOffsetTable::PACKED_LEN, head_record);

    let mut buf = Vec::new();

    write_into(&mut buf, &offset_table);
    write_into(&mut buf, &head_record);

    head.checksum_adjustment = 0xB1B0AFBA -
        util::checksum(&buf).overflowing_add(head_record.checksum).0;

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
