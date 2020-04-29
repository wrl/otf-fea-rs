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

struct TTFTableHeader {
    tag: u32,
    checksum: u32,
    offset_from_start_of_file: u32,
    length: u32
}

#[allow(non_snake_case, non_camel_case_types)]
mod GPOS {
    struct Header_1_0 {
        major: u16,
        minor: u16,
        script_list_offset: u16,
        feature_list_offset: u16,
        lookup_list_offset: u16
    }

    struct Header_1_1 {
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
    struct Header_1_0 {
        major: u16,
        minor: u16,
        glyph_class_def_offset: u16,
        attach_list_offset: u16,
        lig_caret_list_offset: u16,
        mark_attach_class_def_offset: u16
    }

    struct Header_1_2 {
        major: u16,
        minor: u16,
        glyph_class_def_offset: u16,
        attach_list_offset: u16,
        lig_caret_list_offset: u16,
        mark_attach_class_def_offset: u16,
        mark_glyph_sets_def_offset: u16
    }

    struct Header_1_3 {
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

fn main() {
}
