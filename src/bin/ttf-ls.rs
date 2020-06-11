use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use endian_codec::{PackedSize, DecodeBE};

#[macro_use]
extern crate otf_fea_rs;
use otf_fea_rs::compile_model::*;

////
// head table
////

fn display_head(offset_table: &TTFOffsetTable, record: &TTFTableRecord,
    whole_file: &[u8], combined_records_checksum: u32) {
    let head = tables::Head::decode_from_be_bytes(record.table_data(whole_file));

    let directory_end =
        TTFOffsetTable::PACKED_LEN
        + ((offset_table.num_tables as usize) * TTFTableRecord::PACKED_LEN);

    let adjustment =
        0xB1B0AFBAu32.overflowing_sub(
            combined_records_checksum.overflowing_add(
                util::checksum(&whole_file[..directory_end])).0).0;

    println!("checking `head` checksum adjustment against calculated file checksum...");
    println!("    head:       0x{:x}", head.checksum_adjustment);
    println!("    calculated: 0x{:x}", adjustment);
    println!();

    if adjustment == head.checksum_adjustment {
        println!("    good!");
    } else {
        println!("    fail!");
    }
}

////
// name table
////

fn display_name(table_data: &[u8]) {
    let table = tables::Name::decode_from_be_bytes(table_data);
    println!("{:#?}", table);
}

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

fn print_table_record(t: &TTFTableRecord, whole_file: &[u8]) {
    let data = t.table_data(whole_file);
    let calculated_checksum = match t.tag {
        tag!(h,e,a,d) => util::checksum_head(data),
        _ => util::checksum(data)
    };

    let good =
        if t.checksum == calculated_checksum {
            ' '
        } else {
            '!'
        };

    println!("  {}    {: <13}{}{} {: <16}{: <16}",
        t.tag,
        t.checksum,
        good, good,
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

    let mut records = Vec::new();
    let mut head_record = None;
    let mut running_checksum = 0u32;

    for _ in 0..offset_table.num_tables {
        let (record_buf, r) = rest.split_at(TTFTableRecord::PACKED_LEN);
        let record = TTFTableRecord::decode_from_be_bytes(record_buf);
        rest = r;

        print_table_record(&record, &buf);
        records.push(record);

        if record.tag == tag!(h,e,a,d) {
            head_record = Some(records.len() - 1);
        }

        running_checksum = running_checksum.overflowing_add(record.checksum).0;
    }

    println!();

    if let Some(idx) = head_record {
        let record = records[idx];
        display_head(&offset_table, &record, &buf, running_checksum);
    } else {
        println!("no `head` table, skipping file checksum verification");
    }

    println!();

    if let Some(name_record) = records.iter().find(|r| r.tag == tag!(n,a,m,e)) {
        display_name(name_record.table_data(&buf));
    }

    println!();
    Ok(())
}

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    read_ttf(&path).unwrap();
}
