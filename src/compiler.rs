use crate::parse_model as pm;

use endian_codec::{PackedSize, EncodeBE};
use crate::compile_model::*;
use crate::compile_model::util;

use crate::tag;

struct CompilerState {
    pub head_table: Option<tables::Head>,
    tables: Vec<(pm::Tag, Vec<u8>)>
}

impl CompilerState {
    fn new() -> Self {
        Self {
            head_table: None,
            tables: Vec::new(),
        }
    }
}

fn handle_table(ctx: &mut CompilerState, table: &pm::Table) {
    let pm::Table { tag, statements } = table;

    match tag {
        pm::TableTag::head =>
            ctx.head_table = Some(tables::Head::from_parsed_table(statements)),
        pm::TableTag::name => {
            let table = tables::Name::from_parsed_table(statements);
            ctx.tables.push((tag!(n,a,m,e), table.to_be()));
        }

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

fn table_len<T: PackedSize>(_: &T) -> usize {
    return util::align_len(T::PACKED_LEN);
}

fn write_into<T: PackedSize + EncodeBE>(v: &mut Vec<u8>, p: &T) {
    let start = v.len();
    v.resize(util::align_len(start + table_len(p)), 0u8);
    p.encode_as_be_bytes(&mut v[start..]);
}

fn prepare_head(ctx: &mut CompilerState) {
    let mut head = match ctx.head_table {
        Some(ref mut head) => head,
        None => return
    };

    // all stuff to get a clean diff between our output and `spec9c1.ttf`
    head.magic_number = 0;
    head.created = 3406620153.into();
    head.modified = 3647951938.into();
    head.font_direction_hint = 0;

    let mut encoded = vec![0u8; tables::Head::PACKED_LEN];
    head.encode_as_be_bytes(&mut encoded);

    ctx.tables.insert(0, (
        tag!(h,e,a,d),
        encoded
    ));
}

fn actually_compile(ctx: &mut CompilerState, buf: &mut Vec<u8>) {
    prepare_head(ctx);

    let offset_table = TTFOffsetTable::new(
        TTFVersion::TTF, ctx.tables.len() as u16);
    write_into(buf, &offset_table);

    let mut offset = util::align_len(buf.len() +
        (ctx.tables.len() * TTFTableRecord::PACKED_LEN));
    let mut running_checksum = 0u32;

    for (tag, encoded) in ctx.tables.iter() {
        let checksum = util::checksum(encoded);

        let record = TTFTableRecord {
            tag: *tag,
            checksum,
            offset_from_start_of_file: offset as u32,
            length: encoded.len() as u32
        };

        write_into(buf, &record);

        offset += util::align_len(encoded.len());
        running_checksum = running_checksum.overflowing_add(checksum).0;
    }

    buf.resize(util::align_len(buf.len()), 0u8);

    if let Some(ref mut head) = ctx.head_table {
        head.checksum_adjustment = 0xB1B0AFBAu32.overflowing_sub(
            util::checksum(&buf).overflowing_add(running_checksum).0).0;

        head.encode_as_be_bytes(&mut ctx.tables[0].1);
    }

    for (_, encoded) in ctx.tables.iter() {
        buf.extend(encoded.iter());
        buf.resize(util::align_len(buf.len()), 0u8);
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
