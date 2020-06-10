use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use endian_codec::{PackedSize, DecodeBE};

use otf_fea_rs::compile_model::*;

////
// metadata
////

fn print_offset_table(t: &TTFOffsetTable) {
    println!("offset table:");
    println!("    version: {:?}", t.version);
    println!("    num_tables: {}", t.num_tables);
    println!("    search_range: {}", t.search_range);
    println!("    entry_selector: {}", t.entry_selector);
    println!("    range_shift: {}", t.range_shift);
}

fn print_table_record(t: &TTFTableRecord) {
    println!("  {}    {: <16}{: <16}{: <16}",
        t.tag,
        t.checksum,
        t.offset_from_start_of_file,
        t.length);
}

////
// entry point
////

fn read_ttf(path: &str) -> io::Result<()> {
    let mut f = File::open(path)?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    let (offset_buf, mut rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
    let offset_table = TTFOffsetTable::decode_from_be_bytes(offset_buf);
    print_offset_table(&offset_table);

    if let TTFVersion::Unknown(_) = offset_table.version {
        println!("don't know how to read this TTF version");
        return Ok(())
    }

    println!();

    println!("  tag     checksum        offset          length ");
    println!("-----------------------------------------------------");

    for _ in 0..offset_table.num_tables {
        let (record_buf, r) = rest.split_at(TTFTableRecord::PACKED_LEN);
        let record = TTFTableRecord::decode_from_be_bytes(record_buf);

        print_table_record(&record);
        rest = r;
    }

    println!();
    Ok(())
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    read_ttf(&path).unwrap();
}
