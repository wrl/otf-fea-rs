use std::env;
use std::fs::File;

use otf_fea_rs::parser;
use otf_fea_rs::parse_model::*;

use otf_fea_rs::compile_model as cm;

struct CompilerState {
    pub(crate) head: cm::head::Head
}

impl CompilerState {
    fn new() -> Self {
        Self {
            head: cm::head::Head::new()
        }
    }
}

fn handle_head_table(ctx: &mut CompilerState, statements: &[TableStatement]) {
    use TableStatement::*;
    let mut revision: f64 = 0.0;

    for s in statements {
        match s {
            FontRevision(head::FontRevision(f)) => revision = *f,
            _ => unreachable!()
        }
    }

    println!("?? {:#?}", statements);
    ctx.head.font_revision = cm::util::Fixed1616::from_f32(revision as f32);
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

fn main() {
    let path = env::args().skip(1).next()
        .expect("need a path");

    let f = File::open(&path).unwrap();

    let definitions = parser::parse_file(f).unwrap();

    let mut ctx = CompilerState::new();

    for d in definitions {
        handle_top_level(&mut ctx, &d);
    }

    println!("{:#?}\n", ctx.head);
    println!("{:032b}", ctx.head.font_revision.to_bits());
}
