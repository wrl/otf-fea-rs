use endian_codec::{PackedSize, EncodeBE};

use crate::parse_model as pm;

use crate::{
    GlyphOrder,
    tag,
    Tag
};

use crate::compile_model::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::util;


struct CompilerState {
    pub glyph_order: GlyphOrder,

    pub head_table: Option<tables::Head>,
    pub gpos_table: Option<tables::GPOS>,
    tables: Vec<(Tag, Vec<u8>)>
}

impl CompilerState {
    fn new() -> Self {
        Self {
            glyph_order: GlyphOrder::new(),

            head_table: None,
            gpos_table: None,

            tables: Vec::new(),
        }
    }
}

/**
 * feature definitions
 */

use crate::compile_model::LookupSubtable;
use tables::gpos::{
    HasLookups,
    TableWithLookups,
    PairGlyphs,
    PairValueRecord,
};

#[allow(dead_code)]
enum Block<'a> {
    Feature(&'a Tag),
    Lookup(&'a pm::LookupBlockLabel)
}

fn feature_is_vertical(tag: &Tag) -> bool {
    match tag {
        tag!(v,k,r,n) | tag!(v,p,a,l)
            | tag!(v,h,a,l) | tag!(v,a,l,t) => true,

        _ => false
    }
}

impl<'a> Block<'a> {
    fn is_vertical(&self) -> bool {
        match self {
            Block::Feature(tag) => feature_is_vertical(tag),
            Block::Lookup(_) => false,
        }
    }

    fn find_or_insert_lookup<'b, T, L>(&self, table: &'b mut T) -> &'b mut Lookup<L>
        where T: TableWithLookups + HasLookups<Tag> + HasLookups<pm::LookupBlockLabel>,
              L: LookupSubtable<T::Lookup>
    {
        match *self {
            Block::Feature(f) => table.find_or_insert_lookup(f),
            Block::Lookup(l) => table.find_or_insert_lookup(l)
        }
    }
}

fn handle_position_statement(ctx: &mut CompilerState, block: &Block, p: &pm::Position) -> CompileResult<()> {
    use pm::Position::*;

    match p {
        Pair { glyph_classes, value_records } => {
            let gpos = ctx.gpos_table.get_or_insert_with(|| tables::GPOS::new());
            let lookup: &mut Lookup<PairGlyphs> = block.find_or_insert_lookup(gpos);

            if lookup.subtables.len() == 0 {
                lookup.subtables.push(PairGlyphs::new());
            }

            let pair_lookup = &mut lookup.subtables[0];

            let vertical = block.is_vertical();

            for first_glyph in glyph_classes.0.iter_glyphs(&ctx.glyph_order) {
                let pairs = pair_lookup.entry(first_glyph?)
                    .or_default();

                let vr1 = ValueRecord::from_parsed(&value_records.0, vertical);
                let vr2 = value_records.1.as_ref()
                    .map(|vr| ValueRecord::from_parsed(vr, vertical))
                    .unwrap_or_else(|| ValueRecord::zero());

                for second_glyph in glyph_classes.1.iter_glyphs(&ctx.glyph_order) {
                    let second_glyph = second_glyph?;

                    let pvr = PairValueRecord {
                        second_glyph,
                        records: (vr1.clone(), vr2.clone())
                    };

                    pairs.push(pvr);
                }
            }
        },

        _ => panic!()
    };

    Ok(())
}

fn handle_feature_definition(ctx: &mut CompilerState, def: &pm::FeatureDefinition) -> CompileResult<()> {
    use pm::BlockStatement::*;

    let tag = &def.tag;

    println!("feature {}:", tag);

    let block = Block::Feature(tag);

    for s in &def.statements {
        match s {
            Position(pos) => handle_position_statement(ctx, &block, pos)?,
            _ => panic!()
        }
    }

    Ok(())
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

fn handle_top_level(ctx: &mut CompilerState, statement: &pm::TopLevelStatement) -> CompileResult<()> {
    use pm::TopLevelStatement::*;

    match statement {
        Table(ref t) => handle_table(ctx, t),
        FeatureDefinition(ref fd) => handle_feature_definition(ctx, fd)?,
        LanguageSystem(ref _ls) => (),

        s => {
            println!("unhandled {:#?}\n", s);
            panic!();
        }
    }

    Ok(())
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
                .overflowing_sub(
                    whole_file_checksum
                        .overflowing_add(running_checksum).0)
                .0
        };

        head.encode_as_be_bytes(&mut ctx.tables[0].1);
    }

    for (_, encoded) in ctx.tables.iter() {
        buf.extend(encoded.iter());
        buf.resize(util::align_len(buf.len()), 0u8);
    }
}

pub fn compile_iter<'a, I>(glyph_order: GlyphOrder, statements: I, out: &mut Vec<u8>) -> CompileResult<()>
    where I: Iterator<Item = &'a pm::TopLevelStatement>
{
    let mut ctx = CompilerState::new();

    ctx.glyph_order = glyph_order;

    for s in statements {
        handle_top_level(&mut ctx, &s)?;
    }

    // FIXME: "add head table" should be a compiler option
    //
    // if ctx.head_table.is_none() {
    //     ctx.head_table = Some(tables::Head::new());
    // }

    if let Some(gpos) = ctx.gpos_table.as_ref() {
        let mut buf = EncodeBuf::new();
        gpos.ttf_encode(&mut buf).unwrap();

        ctx.tables.push((
            tag!(G,P,O,S),
            buf.bytes
        ));
    }

    actually_compile(&mut ctx, out);

    if let Some(gpos) = ctx.gpos_table.as_ref() {
        println!("{:#?}", gpos);
    }

    Ok(())
}

pub fn compile(glyph_order: GlyphOrder, statements: &[pm::TopLevelStatement],
        out: &mut Vec<u8>) {
    compile_iter(glyph_order, statements.iter(), out).unwrap()
}
