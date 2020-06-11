use endian_codec::{PackedSize, EncodeBE, DecodeBE};

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_0 {
    major: u16,
    minor: u16,
    glyph_class_def_offset: u16,
    attach_list_offset: u16,
    lig_caret_list_offset: u16,
    mark_attach_class_def_offset: u16
}

#[derive(Debug, Copy, Clone, PackedSize, EncodeBE, DecodeBE)]
struct Header_1_2 {
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
    major: u16,
    minor: u16,
    glyph_class_def_offset: u16,
    attach_list_offset: u16,
    lig_caret_list_offset: u16,
    mark_attach_class_def_offset: u16,
    mark_glyph_sets_def_offset: u16,
    item_var_store_offset: u16
}
