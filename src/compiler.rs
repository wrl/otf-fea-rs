use std::iter;

use endian_codec::{PackedSize, EncodeBE};

use crate::*;
use crate::util::*;

use crate::compile_model::*;
use crate::compile_model::util::encode::*;
use crate::compile_model::util;

use crate::parse_model as pm;


/**
 * feature definitions
 */

use crate::compile_model::LookupSubtable;
use tables::gpos::{
    GPOS,
    KeyedLookups,

    Pair,
    PairGlyphs,
    PairValueRecord,

    PairClass,
};

#[allow(dead_code)]
enum Block<'a> {
    Feature(&'a FeatureTag),
    Lookup(&'a pm::LookupName)
}

fn feature_is_vertical(tag: &FeatureTag) -> bool {
    match tag {
        feature_tag!(v,k,r,n)
            | feature_tag!(v,p,a,l)
            | feature_tag!(v,h,a,l)
            | feature_tag!(v,a,l,t) => true,

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

    fn find_or_insert_lookup<'b, T, L, S>(&self, table: &'b mut T) -> &'b mut Lookup<S>
        where T: KeyedLookups<FeatureTag, L> + KeyedLookups<pm::LookupName, L>,
              S: LookupSubtable<L>
    {
        match *self {
            Block::Feature(f) => table.find_or_insert_lookup(f),
            Block::Lookup(l) => table.find_or_insert_lookup(l)
        }
    }

    fn insert_into_script(&self, gpos: &mut GPOS, script_tag: &ScriptTag) {
        let feature_tag = match self {
            Block::Feature(tag) => *tag,
            Block::Lookup(_) => return
        };

        gpos.script_list.script_for_tag_mut(script_tag)
            .default_lang_sys
            .features
            .insert(*feature_tag);
    }

    fn subtable_breaks(&self) -> usize {
        0
    }
}

fn get_subtable_variant<'a, E, T>(lookup: &'a mut Lookup<E>, skip: usize) -> &'a mut T
    where T: VariantExt<E> + Default + Into<E>
{
    let idx = lookup.subtables.iter().enumerate()
        .filter_map(|(idx, subtable)| {
            T::get_variant(subtable)
                .map(|_| idx)
        })
        .skip(skip)
        .next()

        .unwrap_or_else(|| {
            let idx = lookup.subtables.len();
            lookup.subtables.push(T::default().into());
            idx
        });

    T::get_variant_mut(&mut lookup.subtables[idx]).unwrap()
}

fn handle_pair_position_glyphs(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    let pm::position::Pair {
        glyph_classes,
        value_records
    } = pair;

    let gpos = ctx.gpos_table.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<Pair> = block.find_or_insert_lookup(gpos);

    let subtable: &mut PairGlyphs = get_subtable_variant(lookup, block.subtable_breaks());
    let vertical = block.is_vertical();

    for first_glyph in glyph_classes.0.iter_glyphs(&ctx.glyph_order) {
        let pairs = subtable.entry(first_glyph?)
            .or_default();

        let vr1 = ValueRecord::from_parsed(&value_records.0, vertical);
        let vr2 = ValueRecord::from_parsed(&value_records.1, vertical);

        for second_glyph in glyph_classes.1.iter_glyphs(&ctx.glyph_order) {
            let second_glyph = second_glyph?;

            let pvr = PairValueRecord {
                second_glyph,
                records: (vr1.clone(), vr2.clone())
            };

            pairs.push(pvr);
        }
    }

    block.insert_into_script(gpos, &script_tag!(D,F,L,T));
    Ok(())
}

fn handle_pair_position_class(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    let pm::position::Pair {
        glyph_classes,
        value_records
    } = pair;

    let gpos = ctx.gpos_table.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<Pair> = block.find_or_insert_lookup(gpos);

    let vertical = block.is_vertical();

    let classes = (
        ClassDef::from_glyph_class(&glyph_classes.0, &ctx.glyph_order)?,
        ClassDef::from_glyph_class(&glyph_classes.1, &ctx.glyph_order)?
    );

    let value_records = (
        ValueRecord::from_parsed(&value_records.0, vertical),
        ValueRecord::from_parsed(&value_records.1, vertical)
    );

    let mut skip = block.subtable_breaks();

    let subtable = loop {
        let subtable: &mut PairClass = get_subtable_variant(lookup, skip);

        if subtable.can_add_pair(&classes) {
            break subtable;
        } else {
            skip += 1;
        }
    };

    subtable.add_pair(classes, value_records)?;

    Ok(())
}

fn handle_pair_position(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    if pair.glyph_classes.0.is_single() {
        handle_pair_position_glyphs(ctx, block, pair)
    } else {
        handle_pair_position_class(ctx, block, pair)
    }
}

fn handle_position_statement(ctx: &mut CompilerState, block: &Block, p: &pm::Position) -> CompileResult<()> {
    use pm::Position::*;

    match p {
        Pair(pair) => handle_pair_position(ctx, block, pair),
        _ => panic!()
    }
}

fn handle_lookup_reference(ctx: &mut CompilerState, block: &Block, name: &pm::LookupName) -> CompileResult<()> {
    let gpos = match ctx.gpos_table.as_mut() {
        None => return Ok(()),
        Some(gpos) => gpos
    };

    let feature_indices = match block {
        Block::Feature(tag) => gpos.feature_list.indices_for_tag_mut(tag),
        Block::Lookup(_) =>
            panic!("lookup references from inside a lookup block are unsupported")
    };

    let lookup_indices = gpos.named_lookups.get(name)
        .map(|indices| Either2::A(indices.iter()))
        .unwrap_or_else(|| Either2::B(iter::empty::<&u16>()));

    for idx in lookup_indices {
        feature_indices.push(*idx);
    }

    block.insert_into_script(gpos, &script_tag!(D,F,L,T));
    Ok(())
}

fn handle_block_statements(ctx: &mut CompilerState, block: &Block, statements: &[pm::BlockStatement]) -> CompileResult<()> {
    use pm::BlockStatement::*;

    for s in statements {
        match s {
            Position(pos) => handle_position_statement(ctx, block, pos)?,
            Lookup(pm::Lookup(name)) => handle_lookup_reference(ctx, block, name)?,

            stmt => panic!("unimplemented block statement {:?}", stmt)
        }
    }

    Ok(())
}

fn handle_feature_definition(ctx: &mut CompilerState, def: &pm::FeatureDefinition) -> CompileResult<()> {
    let tag = &def.tag;
    let block = Block::Feature(tag);

    println!("feature {}:", tag);

    handle_block_statements(ctx, &block, &def.statements)
}

fn handle_lookup_definition(ctx: &mut CompilerState, def: &pm::LookupDefinition) -> CompileResult<()> {
    let name = &def.label;
    let block = Block::Lookup(name);

    println!("lookup {}:", name);

    handle_block_statements(ctx, &block, &def.statements)
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
            ctx.tables_encoded.push((tag!(n,a,m,e), table.to_be()));
        }

        _ => panic!()
    }
}

fn handle_top_level(ctx: &mut CompilerState, statement: &pm::TopLevelStatement) -> CompileResult<()> {
    use pm::TopLevelStatement::*;

    match statement {
        LanguageSystem(ref _ls) => (),

        Table(ref t) => handle_table(ctx, t),

        FeatureDefinition(ref fd) => handle_feature_definition(ctx, fd)?,
        LookupDefinition(ref ld) => handle_lookup_definition(ctx, ld)?,

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

    ctx.tables_encoded.insert(0, (
        tag!(h,e,a,d),
        encoded
    ));
}

fn actually_compile(ctx: &mut CompilerState, buf: &mut Vec<u8>) {
    prepare_head(ctx);

    let offset_table = TTFOffsetTable::new(
        TTFVersion::TTF, ctx.tables_encoded.len() as u16);
    write_into(buf, &offset_table);

    let mut offset = util::align_len(buf.len() +
        (ctx.tables_encoded.len() * TTFTableRecord::PACKED_LEN));
    let mut running_checksum = 0u32;

    for (tag, encoded) in ctx.tables_encoded.iter() {
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

        head.encode_as_be_bytes(&mut ctx.tables_encoded[0].1);
    }

    for (_, encoded) in ctx.tables_encoded.iter() {
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

        ctx.tables_encoded.push((
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
