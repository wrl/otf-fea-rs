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

use crate::compile_model::lookup::*;
use tables::{
    gsub,
    gpos
};

struct Block<'a> {
    ident: BlockIdent<'a>,
    subtable_breaks: usize
}

impl<'a> Block<'a> {
    pub fn new_feature(tag: &'a FeatureTag) -> Self {
        Self {
            ident: BlockIdent::Feature(tag),
            subtable_breaks: 0
        }
    }

    pub fn new_lookup(name: &'a pm::LookupName) -> Self {
        Self {
            ident: BlockIdent::Lookup(name),
            subtable_breaks: 0
        }
    }

    pub fn add_subtable_break(&mut self) {
        self.subtable_breaks += 1;
    }
}

enum BlockIdent<'a> {
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
        match self.ident {
            BlockIdent::Feature(tag) => feature_is_vertical(tag),
            BlockIdent::Lookup(_) => false,
        }
    }

    fn find_or_insert_lookup<'b, T, L, S>(&self, table: &'b mut T) -> &'b mut Lookup<S>
        where T: KeyedLookups<FeatureTag, L> + KeyedLookups<pm::LookupName, L>,
              S: LookupSubtable<L>
    {
        match self.ident {
            BlockIdent::Feature(f) => table.find_or_insert_lookup(f),
            BlockIdent::Lookup(l) => table.find_or_insert_lookup(l)
        }
    }

    fn insert_into_script<T>(&self, table: &mut LookupTable<T>, script_tag: &ScriptTag) {
        let feature_tag = match self.ident {
            BlockIdent::Feature(tag) => tag,
            BlockIdent::Lookup(_) => return
        };

        table.script_list.script_for_tag_mut(script_tag)
            .default_lang_sys
            .features
            .insert(*feature_tag);
    }
}

fn handle_pair_position_glyphs(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    let pm::position::Pair {
        glyph_classes,
        value_records
    } = pair;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Pair> = block.find_or_insert_lookup(gpos);

    let subtable: &mut gpos::PairGlyphs = lookup.get_subtable_variant(block.subtable_breaks);
    let vertical = block.is_vertical();

    for first_glyph in glyph_classes.0.iter_glyphs(&ctx.glyph_order) {
        let pairs = subtable.entry(first_glyph?)
            .or_default();

        let vr1 = ValueRecord::from_parsed(&value_records.0, vertical);
        let vr2 = ValueRecord::from_parsed(&value_records.1, vertical);

        for second_glyph in glyph_classes.1.iter_glyphs(&ctx.glyph_order) {
            let second_glyph = second_glyph?;

            let pvr = gpos::PairValueRecord {
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

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Pair> = block.find_or_insert_lookup(gpos);

    let vertical = block.is_vertical();

    let classes = (
        ClassDef::from_glyph_class(&glyph_classes.0, &ctx.glyph_order)?,
        ClassDef::from_glyph_class(&glyph_classes.1, &ctx.glyph_order)?
    );

    let value_records = (
        ValueRecord::from_parsed(&value_records.0, vertical),
        ValueRecord::from_parsed(&value_records.1, vertical)
    );

    let mut skip = block.subtable_breaks;

    let subtable = loop {
        let subtable: &mut gpos::PairClass = lookup.get_subtable_variant(skip);

        if subtable.can_add_pair(&classes) {
            break subtable;
        } else {
            skip += 1;
        }
    };

    subtable.add_pair(classes, value_records)?;

    block.insert_into_script(gpos, &script_tag!(D,F,L,T));
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

fn handle_multiple_substitution(ctx: &mut CompilerState, block: &Block, sub: &pm::substitute::Multiple) -> CompileResult<()> {
    let glyph = ctx.glyph_order.id_for_glyph(&sub.glyph)?;

    let sequence: Vec<_> = sub.sequence.iter()
        .map(|gr| ctx.glyph_order.id_for_glyph(gr))
        .collect::<Result<_, _>>()?;

    let gsub = ctx.gsub.get_or_insert_with(|| tables::GSUB::new());
    let lookup: &mut Lookup<gsub::Multiple> = block.find_or_insert_lookup(gsub);

    let subtable = lookup.get_subtable(block.subtable_breaks);

    // FIXME: find next subtable if sequence is already in this one?
    //        overwrite as we're doing now?
    subtable.insert(glyph, sequence);

    block.insert_into_script(gsub, &script_tag!(D,F,L,T));
    Ok(())
}

fn handle_alternate_substitution(ctx: &mut CompilerState, block: &Block, sub: &pm::substitute::Alternate) -> CompileResult<()> {
    let glyph = ctx.glyph_order.id_for_glyph(&sub.glyph)?;

    let replacement: Vec<_> =
        sub.replacement.iter_glyphs(&ctx.glyph_order)
        .collect::<Result<_, _>>()?;

    let gsub = ctx.gsub.get_or_insert_with(|| tables::GSUB::new());
    let lookup: &mut Lookup<gsub::Alternate> = block.find_or_insert_lookup(gsub);

    let subtable = lookup.get_subtable(block.subtable_breaks);

    // FIXME: find next subtable if sequence is already in this one?
    //        overwrite as we're doing now?
    subtable.insert(glyph, replacement);

    block.insert_into_script(gsub, &script_tag!(D,F,L,T));
    Ok(())
}

fn handle_substitute_statement(ctx: &mut CompilerState, block: &Block, s: &pm::Substitute) -> CompileResult<()> {
    use pm::Substitute::*;

    match s {
        Multiple(m) => handle_multiple_substitution(ctx, block, m),
        Alternate(a) => handle_alternate_substitution(ctx, block, a),

        s => panic!("{:#?}", s)
    }
}

fn handle_lookup_reference(ctx: &mut CompilerState, block: &Block, name: &pm::LookupName) -> CompileResult<()> {
    let gpos = match ctx.gpos.as_mut() {
        None => return Ok(()),
        Some(gpos) => gpos
    };

    let feature_indices = match block.ident {
        BlockIdent::Feature(tag) => gpos.feature_list.indices_for_tag_mut(tag),
        BlockIdent::Lookup(_) =>
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

fn handle_block_statements(ctx: &mut CompilerState, block: &mut Block, statements: &[pm::BlockStatement]) -> CompileResult<()> {
    use pm::BlockStatement::*;

    for s in statements {
        match s {
            Position(pos) => handle_position_statement(ctx, block, pos)?,
            Substitute(sub) => handle_substitute_statement(ctx, block, sub)?,

            Lookup(pm::Lookup(name)) => handle_lookup_reference(ctx, block, name)?,

            Subtable => block.add_subtable_break(),

            stmt => panic!("unimplemented block statement {:?}", stmt)
        }
    }

    Ok(())
}

fn handle_feature_definition(ctx: &mut CompilerState, def: &pm::FeatureDefinition) -> CompileResult<()> {
    let tag = &def.tag;
    let mut block = Block::new_feature(tag);

    println!("feature {}:", tag);

    handle_block_statements(ctx, &mut block, &def.statements)
}

fn handle_lookup_definition(ctx: &mut CompilerState, def: &pm::LookupDefinition) -> CompileResult<()> {
    let name = &def.label;
    let mut block = Block::new_lookup(name);

    println!("lookup {}:", name);

    handle_block_statements(ctx, &mut block, &def.statements)
}

/**
 * simple top level
 */

fn handle_table(ctx: &mut CompilerState, table: &pm::Table) {
    let pm::Table { tag, statements } = table;

    match tag {
        pm::TableTag::head =>
            ctx.head = Some(tables::Head::from_parsed_table(statements)),
        pm::TableTag::name => {
            let table = tables::Name::from_parsed_table(statements);
            ctx.tables_encoded.push((tag!(n,a,m,e), table.to_be()));
        }

        _ => panic!()
    }
}

fn handle_mark_class_statement(ctx: &mut CompilerState, mark_class: &pm::MarkClass) -> CompileResult<()> {
    if !ctx.mark_class_statements_allowed {
        return Err(CompileError::MarkClassNotAllowed);
    }

    let pm::MarkClass {
        glyph_class,
        anchor,
        class_name
    } = mark_class;

    ctx.mark_class_table.entry(class_name.clone())
        .or_default()
        .push((glyph_class.clone(), anchor.clone()));

    Ok(())
}

fn handle_top_level(ctx: &mut CompilerState, statement: &pm::TopLevelStatement) -> CompileResult<()> {
    use pm::TopLevelStatement::*;

    match statement {
        LanguageSystem(ref _ls) => (),

        Table(ref t) => handle_table(ctx, t),

        FeatureDefinition(ref fd) => handle_feature_definition(ctx, fd)?,
        LookupDefinition(ref ld) => handle_lookup_definition(ctx, ld)?,

        MarkClass(ref mc) => handle_mark_class_statement(ctx, mc)?,

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
    let mut head = match ctx.head {
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

    if let Some(ref mut head) = ctx.head {
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

    macro_rules! encode_table {
        ($table:ident, $tag:expr) => {
            if let Some(table) = ctx.$table.as_ref() {
                let mut buf = EncodeBuf::new();
                table.ttf_encode(&mut buf).unwrap();

                ctx.tables_encoded.push((
                    $tag,
                    buf.bytes
                ));
            }
        }
    }

    encode_table!(gpos, tag!(G,P,O,S));
    encode_table!(gsub, tag!(G,S,U,B));

    actually_compile(&mut ctx, out);

    if let Some(gpos) = ctx.gpos.as_ref() {
        println!("{:#?}", gpos);
    }

    if let Some(gsub) = ctx.gsub.as_ref() {
        println!("{:#?}", gsub);
    }

    Ok(())
}

pub fn compile(glyph_order: GlyphOrder, statements: &[pm::TopLevelStatement],
        out: &mut Vec<u8>) {
    compile_iter(glyph_order, statements.iter(), out).unwrap()
}
