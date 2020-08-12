use crate::parse_model as pm;

use endian_codec::{PackedSize, EncodeBE};
use crate::compile_model::*;
use crate::compile_model::util;

use crate::tag;

struct CompilerState {
    pub head_table: Option<tables::Head>,
    pub gpos_table: Option<tables::GPOS>,
    tables: Vec<(pm::Tag, Vec<u8>)>
}

impl CompilerState {
    fn new() -> Self {
        Self {
            head_table: None,
            gpos_table: None,
            tables: Vec::new(),
        }
    }

    fn get_gpos(&mut self) -> &mut tables::GPOS {
        self.gpos_table
            .get_or_insert_with(|| tables::GPOS::new())
    }
}

/**
 * feature definitions
 */

use tables::gpos::GPOSSubtable;

fn find_lookup<'a>(gpos: &'a tables::GPOS, feature_tag: &pm::Tag, lookup_type: u16) -> Option<usize>
{
    let indices = gpos.feature_list.indices_for_tag(feature_tag);

    for i in indices {
        let i = *i as usize;

        match gpos.lookup_list.0.get(i) {
            Some(Lookup { lookup_type: lt, .. })
                if *lt == lookup_type =>
                    return Some(i),

            _ => continue
        }
    }

    None
}

fn find_or_insert_lookup<'a>(ctx: &'a mut CompilerState, feature_tag: &pm::Tag, lookup_type: u16)
        -> &'a mut Lookup<GPOSSubtable> {
    let gpos = ctx.get_gpos();

    let idx = match find_lookup(gpos, feature_tag, lookup_type) {
        Some(idx) => idx,
        None => {
            let indices = gpos.feature_list.indices_for_tag_mut(feature_tag);
            let idx = gpos.lookup_list.0.len();

            indices.push(idx as u16);
            gpos.lookup_list.0.push(Lookup::new(lookup_type));

            idx
        }
    };

    &mut gpos.lookup_list.0[idx]
}

fn handle_position_statement(ctx: &mut CompilerState, feature_tag: &pm::Tag, p: &pm::Position) {
    use pm::Position::*;

    match p {
        Pair { glyph_classes, value_records } => {
            // FIXME: type 2 is pairpos
            let _lookup = find_or_insert_lookup(ctx, feature_tag, 2);
        },

        _ => (),
    };
}

fn handle_feature_definition(ctx: &mut CompilerState, def: &pm::FeatureDefinition) {
    use pm::BlockStatement::*;

    let tag = &def.tag;

    println!("feature {}:", tag);

    for s in &def.statements {
        match s {
            Position(pos) => handle_position_statement(ctx, tag, pos),
            _ => ()
        }
    }
}

/**
 * simple top level
 */

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
        FeatureDefinition(ref fd) => handle_feature_definition(ctx, fd),

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
        head.checksum_adjustment = {
            let whole_file_checksum = util::checksum(&buf);

            0xB1B0AFBAu32
                .overflowing_sub(whole_file_checksum).0
                .overflowing_add(running_checksum).0
        };

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

    if let Some(gpos) = ctx.gpos_table.as_ref() {
        println!("{:#?}", gpos);
    }
}

pub fn compile(statements: &[pm::TopLevelStatement], out: &mut Vec<u8>) {
    compile_iter(statements.iter(), out)
}
