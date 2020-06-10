use crate::parse_model::*;

use endian_codec::{PackedSize, EncodeBE};
use crate::compile_model as cm;
use crate::compile_model::util;

struct CompilerState {
    pub head: Option<cm::head::Head>
}

impl CompilerState {
    fn new() -> Self {
        Self {
            head: None,
        }
    }
}

fn handle_head_table(ctx: &mut CompilerState, statements: &[TableStatement]) {
    ctx.head = Some(cm::head::Head::from_parsed_table(statements));
}

fn handle_table(ctx: &mut CompilerState, table: &Table) {
    let Table { tag, statements } = table;

    match tag {
        TableTag::head => handle_head_table(ctx, statements),
        _ => panic!()
    }
}

fn handle_top_level(ctx: &mut CompilerState, statement: &TopLevelStatement) {
    use TopLevelStatement::*;

    match statement {
        Table(ref t) => handle_table(ctx, t),

        s => println!("unhandled {:#?}\n", s),
    }
}

/**
 * todo: move this out into a separate file
 */

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
    offset_from_start_of_file: usize, p: &T) -> cm::TTFTableRecord {
    cm::TTFTableRecord {
        tag,
        checksum: checksum_any(p),
        offset_from_start_of_file: align_len(offset_from_start_of_file
            + cm::TTFTableRecord::PACKED_LEN) as u32,
        length: T::PACKED_LEN as u32
    }
}

fn write_into<T: PackedSize + EncodeBE>(v: &mut Vec<u8>, p: &T) {
    let start = v.len();
    v.resize(start + table_len(p), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

fn actually_compile(ctx: &mut CompilerState, buf: &mut Vec<u8>) {
    let offset_table = cm::TTFOffsetTable {
        version: cm::TTFVersion::TTF,
        num_tables: 1,
        search_range: 16,
        entry_selector: 0,
        range_shift: 0
    };

    let mut head = ctx.head.unwrap_or_else(|| cm::head::Head::new());

    // all stuff to get a clean diff between our output and `spec9c1.ttf`
    head.magic_number = 0;
    head.created = 3406620153.into();
    head.modified = 3647951938.into();
    head.font_direction_hint = 0;

    let hdr = record_for(Tag::from_bytes(b"head").unwrap(),
        cm::TTFOffsetTable::PACKED_LEN,
        &head);

    write_into(buf, &offset_table);
    write_into(buf, &hdr);

    head.checksum_adjustment = 0xB1B0AFBAu32.overflowing_sub(
        util::checksum(&buf).overflowing_add(hdr.checksum).0).0;

    write_into(buf, &head);
}

pub fn compile_iter<'a, I>(statements: I, out: &mut Vec<u8>)
    where I: Iterator<Item = &'a TopLevelStatement>
{
    let mut ctx = CompilerState::new();

    for s in statements {
        handle_top_level(&mut ctx, &s);
    }

    actually_compile(&mut ctx, out);
}

pub fn compile(statements: &[TopLevelStatement], out: &mut Vec<u8>) {
    compile_iter(statements.iter(), out)
}
