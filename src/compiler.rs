use crate::parse_model as pm;

use endian_codec::{PackedSize, EncodeBE};
use crate::compile_model::*;
use crate::compile_model::util;

use crate::tag;

struct CompilerTables {
    pub head: Option<tables::Head>,
    pub name: Option<tables::Name>
}

struct CompilerState {
    tables: CompilerTables
}

impl CompilerState {
    fn new() -> Self {
        Self {
            tables: CompilerTables {
                head: None,
                name: None
            }
        }
    }
}

fn handle_table(ctx: &mut CompilerState, table: &pm::Table) {
    let pm::Table { tag, statements } = table;

    match tag {
        pm::TableTag::head =>
            ctx.tables.head = Some(tables::Head::from_parsed_table(statements)),
        pm::TableTag::name =>
            ctx.tables.name = Some(tables::Name::from_parsed_table(statements)),

        _ => panic!()
    }
}

fn handle_top_level(ctx: &mut CompilerState, statement: &pm::TopLevelStatement) {
    use pm::TopLevelStatement::*;

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

fn record_for<T: PackedSize + EncodeBE>(tag: pm::Tag,
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
    v.resize(align_len(start + table_len(p)), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

fn actually_compile(ctx: &mut CompilerState, buf: &mut Vec<u8>) {
    if let Some(ref mut head) = ctx.tables.head {
        // all stuff to get a clean diff between our output and `spec9c1.ttf`
        head.magic_number = 0;
        head.created = 3406620153.into();
        head.modified = 3647951938.into();
        head.font_direction_hint = 0;

        let hdr = record_for(tag!(h,e,a,d), TTFOffsetTable::PACKED_LEN, head);

        let offset_table = TTFOffsetTable::new(TTFVersion::TTF, 1);
        write_into(buf, &offset_table);
        write_into(buf, &hdr);

        head.checksum_adjustment = 0xB1B0AFBAu32.overflowing_sub(
            util::checksum(&buf).overflowing_add(hdr.checksum).0).0;

        write_into(buf, head);
    } else if let Some(ref name) = ctx.tables.name {
        let name_encoded = name.to_be();

        let offset_table = TTFOffsetTable::new(TTFVersion::TTF, 1);
        let record = TTFTableRecord {
            tag: tag!(n,a,m,e),
            checksum: util::checksum(&name_encoded),
            offset_from_start_of_file: align_len(
                TTFOffsetTable::PACKED_LEN
                + TTFTableRecord::PACKED_LEN) as u32,
            length: name_encoded.len() as u32
        };

        write_into(buf, &offset_table);
        write_into(buf, &record);
        buf.extend(&name_encoded);
    } else {
        let offset_table = TTFOffsetTable::new(TTFVersion::TTF, 0);
        write_into(buf, &offset_table);
    }
}

pub fn compile_iter<'a, I>(statements: I, out: &mut Vec<u8>)
    where I: Iterator<Item = &'a pm::TopLevelStatement>
{
    let mut ctx = CompilerState::new();

    for s in statements {
        handle_top_level(&mut ctx, &s);
    }

    actually_compile(&mut ctx, out);
}

pub fn compile(statements: &[pm::TopLevelStatement], out: &mut Vec<u8>) {
    compile_iter(statements.iter(), out)
}
