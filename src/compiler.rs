use std::convert::TryInto;
use std::iter;

use crate::*;
use crate::glyph_class::*;
use crate::util::*;

use crate::compile_model::*;
use crate::compile_model::util::encode::*;

use crate::parse_model as pm;


/**
 * utilities
 */

#[inline]
fn feature_is_vertical(tag: &FeatureTag) -> bool {
    match tag {
        feature_tag!(v,k,r,n)
            | feature_tag!(v,p,a,l)
            | feature_tag!(v,h,a,l)
            | feature_tag!(v,a,l,t) => true,

        _ => false
    }
}

/**
 * block/scope
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

/**
 * feature definitions
 */

fn handle_single_adjustment_position(ctx: &mut CompilerState, block: &Block,
    pos: &pm::position::SingleAdjustment) -> CompileResult<()> {

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Single> = block.find_or_insert_lookup(gpos);

    let vr = ValueRecord::from_parsed(&pos.value_record, block.is_vertical())?;

    if let Some(glyph) = pos.glyph_class.as_single() {
        let glyph_id = ctx.glyph_order.id_for_glyph(glyph)?;

        let subtable = lookup.get_subtable_variant_filter(block.subtable_breaks,
            |subtable: &gpos::SingleArray| subtable.can_add(glyph_id, &vr),
            || gpos::SingleArray::new_with_value_format(
                vr.smallest_possible_format()));

        subtable.add_glyph(glyph_id, vr);
    } else {
        let subtable = lookup.get_subtable_variant_filter(block.subtable_breaks,
            |_: &gpos::SingleClass| false,
            || gpos::SingleClass::new(vr).into());

        for glyph in pos.glyph_class.iter_glyphs(&ctx.glyph_order, &ctx.glyph_class_table) {
            subtable.add_glyph(glyph?);
        }
    }

    Ok(())
}

fn handle_pair_position_glyphs(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    let pm::position::Pair {
        glyph_classes,
        value_records
    } = pair;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Pair> = block.find_or_insert_lookup(gpos);

    let vertical = block.is_vertical();

    let vr1 = ValueRecord::from_parsed(&value_records.0, vertical)?;
    let vr2 = ValueRecord::from_parsed(&value_records.1, vertical)?;

    let value_formats = (
        vr1.smallest_possible_format(),
        vr2.smallest_possible_format()
    );

    let subtable = lookup.get_subtable_variant_filter(block.subtable_breaks,
        |pg: &gpos::PairGlyphs| pg.value_formats_match(&value_formats),
        || gpos::PairGlyphs::new_with_value_formats(value_formats));

    for first_glyph in glyph_classes.0.iter_glyphs(&ctx.glyph_order, &ctx.glyph_class_table) {
        let pairs = subtable.entry(first_glyph?)
            .or_default();

        for second_glyph in glyph_classes.1.iter_glyphs(&ctx.glyph_order, &ctx.glyph_class_table) {
            let second_glyph = second_glyph?;

            let pvr = gpos::PairValueRecord {
                second_glyph,
                records: (vr1.clone(), vr2.clone())
            };

            pairs.push(pvr);
        }
    }

    Ok(())
}

fn handle_pair_position_class(ctx: &mut CompilerState, block: &Block, pair: &pm::position::Pair) -> CompileResult<()> {
    let pm::position::Pair {
        glyph_classes,
        value_records
    } = pair;

    let vertical = block.is_vertical();

    let classes = (
        ClassDef::from_glyph_class(&glyph_classes.0, &ctx.glyph_order, &ctx.glyph_class_table)?,
        ClassDef::from_glyph_class(&glyph_classes.1, &ctx.glyph_order, &ctx.glyph_class_table)?
    );

    let value_records = (
        ValueRecord::from_parsed(&value_records.0, vertical)?,
        ValueRecord::from_parsed(&value_records.1, vertical)?
    );

    let mut skip = block.subtable_breaks;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Pair> = block.find_or_insert_lookup(gpos);

    let subtable = loop {
        let subtable: &mut gpos::PairClass = lookup.get_subtable_variant(skip);

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

fn handle_cursive_position(ctx: &mut CompilerState, block: &Block, cursive: &pm::position::Cursive) -> CompileResult<()> {
    let pm::position::Cursive {
        glyph_class,
        entry,
        exit
    } = cursive;

    let entry: gpos::Anchor = ctx.lookup_anchor(entry)?;
    let exit: gpos::Anchor = ctx.lookup_anchor(exit)?;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::Cursive> = block.find_or_insert_lookup(gpos);
    let subtable = lookup.get_subtable(block.subtable_breaks);

    for glyph_id in glyph_class.iter_glyphs(&ctx.glyph_order, &ctx.glyph_class_table) {
        subtable.add_rule(glyph_id?, entry.clone(), exit.clone());
    }

    Ok(())
}

fn handle_mark_to_base_position(ctx: &mut CompilerState, block: &Block, m2b: &pm::position::MarkToBase) -> CompileResult<()> {
    ctx.mark_class_statements_allowed = false;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::MarkToBase> = block.find_or_insert_lookup(gpos);
    let subtable = lookup.get_subtable(block.subtable_breaks);

    for (anchor, mark_class_name) in &m2b.marks {
        let mark_class = ctx.mark_class_table.get(mark_class_name)
            .ok_or_else(|| CompileError::UnknownMarkClass(mark_class_name.into()))?;

        subtable.add_mark_class(&ctx.glyph_order, &ctx.glyph_class_table, &m2b.base,
            &anchor.try_into()?, mark_class_name, mark_class)?;
    }

    Ok(())
}

fn handle_mark_to_mark_position(ctx: &mut CompilerState, block: &Block, m2m: &pm::position::MarkToMark) -> CompileResult<()> {
    ctx.mark_class_statements_allowed = false;

    let pm::position::MarkToMark {
        marks,
        ..
    } = m2m;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    let lookup: &mut Lookup<gpos::MarkToMark> = block.find_or_insert_lookup(gpos);
    let _subtable = lookup.get_subtable(block.subtable_breaks);

    for (anchor, mark_class_name) in marks {
        let mark_class = ctx.mark_class_table.get(mark_class_name)
            .ok_or_else(|| CompileError::UnknownMarkClass(mark_class_name.into()))?;

        println!("{:?} {:#?}", anchor, mark_class);
    }

    panic!("unimplemented");
    // Ok(())
}

fn handle_position_statement(ctx: &mut CompilerState, block: &Block, p: &pm::Position) -> CompileResult<()> {
    use pm::Position::*;

    let gpos = ctx.gpos.get_or_insert_with(|| tables::GPOS::new());
    block.insert_into_script(gpos, &script_tag!(D,F,L,T));

    match p {
        SingleAdjustment(adj) => handle_single_adjustment_position(ctx, block, adj),

        Pair(pair) => handle_pair_position(ctx, block, pair),
        Cursive(cursive) => handle_cursive_position(ctx, block, cursive),

        MarkToBase(m2b) => handle_mark_to_base_position(ctx, block, m2b),
        MarkToMark(m2m) => handle_mark_to_mark_position(ctx, block, m2m),


        p => panic!("unhandled position statement: {:#?}", p)
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

    Ok(())
}

fn handle_alternate_substitution(ctx: &mut CompilerState, block: &Block, sub: &pm::substitute::Alternate) -> CompileResult<()> {
    let glyph = ctx.glyph_order.id_for_glyph(&sub.glyph)?;

    let replacement: Vec<_> =
        sub.replacement.iter_glyphs(&ctx.glyph_order, &ctx.glyph_class_table)
        .collect::<Result<_, _>>()?;

    let gsub = ctx.gsub.get_or_insert_with(|| tables::GSUB::new());
    let lookup: &mut Lookup<gsub::Alternate> = block.find_or_insert_lookup(gsub);

    let subtable = lookup.get_subtable(block.subtable_breaks);

    // FIXME: find next subtable if sequence is already in this one?
    //        overwrite as we're doing now?
    subtable.insert(glyph, replacement);

    Ok(())
}

fn handle_substitute_statement(ctx: &mut CompilerState, block: &Block, s: &pm::Substitute) -> CompileResult<()> {
    use pm::Substitute::*;

    let gsub = ctx.gsub.get_or_insert_with(|| tables::GSUB::new());
    block.insert_into_script(gsub, &script_tag!(D,F,L,T));

    match s {
        Multiple(m) => handle_multiple_substitution(ctx, block, m),
        Alternate(a) => handle_alternate_substitution(ctx, block, a),

        Single(_) => Ok(())
    }
}

fn handle_lookup_reference(ctx: &mut CompilerState, block: &Block, name: &pm::LookupName) -> CompileResult<()> {
    // FIXME: also needs to happen for gsub

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

            LookupDefinition(ref ld) => handle_lookup_definition(ctx, ld)?,
            NamedGlyphClass(ref gc) => handle_glyph_class_definition(ctx, gc)?,
            MarkClass(ref mc) => handle_mark_class_statement(ctx, mc)?,

            Script(_) | Language(_) | FeatureNames(_) => {},

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
            // let table = tables::Name::from_parsed_table(statements);
            // ctx.tables_encoded.push((tag!(n,a,m,e), table.to_be()));
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
        .push((glyph_class.clone(), anchor.try_into()?));

    Ok(())
}

fn handle_anchor_definition(ctx: &mut CompilerState, anchor_def: &pm::AnchorDefinition) -> CompileResult<()> {
    let pm::AnchorDefinition {
        name,
        anchor
    } = anchor_def;

    ctx.anchor_table.entry(name.clone())
        .or_insert(anchor.try_into()?);

    Ok(())
}

fn handle_glyph_class_definition(ctx: &mut CompilerState, cls: &NamedGlyphClass) -> CompileResult<()> {
    ctx.glyph_class_table.insert(cls.name.clone(), cls.glyph_class.clone());

    Ok(())
}

fn handle_top_level(ctx: &mut CompilerState, statement: &pm::TopLevelStatement) -> CompileResult<()> {
    use pm::TopLevelStatement::*;

    match statement {
        LanguageSystem(ref _ls) => (),

        Table(ref t) => handle_table(ctx, t),

        FeatureDefinition(ref fd) => handle_feature_definition(ctx, fd)?,
        LookupDefinition(ref ld) => handle_lookup_definition(ctx, ld)?,
        AnchorDefinition(ref ad) => handle_anchor_definition(ctx, ad)?,
        NamedGlyphClass(ref gc) => handle_glyph_class_definition(ctx, gc)?,

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

impl CompilerOutput {
    pub fn merge_encoded_tables(&self, tables: &mut EncodedTables) -> EncodeResult<()> {
        macro_rules! encode_table {
            ($table:ident, $tag:expr) => {
                if let Some(table) = self.$table.as_ref() {
                    let mut buf = EncodeBuf::new_with_glyph_order(&self.glyph_order);
                    table.ttf_encode(&mut buf)?;

                    tables.add_table($tag, buf.bytes, buf.source_map);
                }
            }
        }

        encode_table!(gpos, tag!(G,P,O,S));
        encode_table!(gsub, tag!(G,S,U,B));

        Ok(())
    }

    pub fn encode_tables(&self) -> EncodeResult<EncodedTables> {
        let mut encoded = EncodedTables::new(self.head.clone());
        self.merge_encoded_tables(&mut encoded)?;
        Ok(encoded)
    }
}

pub fn compile_iter<'a, I>(glyph_order: GlyphOrder, statements: I)
    -> CompileResult<CompilerOutput>
    where I: Iterator<Item = &'a pm::TopLevelStatement>
{
    let mut ctx = CompilerState::new();

    ctx.glyph_order = glyph_order;

    for s in statements {
        handle_top_level(&mut ctx, &s)?;
    }

    Ok(ctx.into())
}

#[inline]
pub fn compile(glyph_order: GlyphOrder, statements: &[pm::TopLevelStatement])
    -> CompileResult<CompilerOutput> {
    compile_iter(glyph_order, statements.iter())
}
