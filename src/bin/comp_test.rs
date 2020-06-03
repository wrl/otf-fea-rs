#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::prelude::*;

use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct SFNTHeader {
    // 0x00010000 for ttf, "OTTO" for otf
    version: u32,

    num_tables: u16,

    // (maximum power of 2 <= num_tables) * 16
    search_range: u16,

    // log2(maximum power of 2 <= num_tables)
    entry_selector: u16,

    // (num_tables * 16) - search_range
    range_shift: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct TTFTableHeader {
    tag: u32,
    checksum: u32,
    offset_from_start_of_file: u32,
    length: u32
}

#[allow(non_snake_case, non_camel_case_types)]
mod GPOS {
    use endian_codec::{PackedSize, EncodeBE, DecodeBE};

    #[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
    struct Header_1_0 {
        header: super::TTFTableHeader,

        major: u16,
        minor: u16,
        script_list_offset: u16,
        feature_list_offset: u16,
        lookup_list_offset: u16
    }

    #[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
    struct Header_1_1 {
        header: super::TTFTableHeader,

        major: u16,
        minor: u16,
        script_list_offset: u16,
        feature_list_offset: u16,
        lookup_list_offset: u16,
        feature_variations_offset: u16
    }
}

#[allow(non_snake_case, non_camel_case_types)]
mod GDEF {
    use endian_codec::{PackedSize, EncodeBE, DecodeBE};

    #[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
    struct Header_1_0 {
        header: super::TTFTableHeader,

        major: u16,
        minor: u16,
        glyph_class_def_offset: u16,
        attach_list_offset: u16,
        lig_caret_list_offset: u16,
        mark_attach_class_def_offset: u16
    }

    #[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
    struct Header_1_2 {
        header: super::TTFTableHeader,

        major: u16,
        minor: u16,
        glyph_class_def_offset: u16,
        attach_list_offset: u16,
        lig_caret_list_offset: u16,
        mark_attach_class_def_offset: u16,
        mark_glyph_sets_def_offset: u16
    }

    #[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
    struct Header_1_3 {
        header: super::TTFTableHeader,

        major: u16,
        minor: u16,
        glyph_class_def_offset: u16,
        attach_list_offset: u16,
        lig_caret_list_offset: u16,
        mark_attach_class_def_offset: u16,
        mark_glyph_sets_def_offset: u16,
        item_var_store_offset: u16
    }
}

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
