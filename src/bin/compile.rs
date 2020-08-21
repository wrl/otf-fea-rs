use std::env;
use std::fs::File;
use std::io::prelude::*;

use otf_fea_rs::{
    GlyphOrder,
    IntoGlyphOrder,
    parser,
    compiler
};

fn fealib_builder_glyph_order() -> GlyphOrder {
    let glyphs = [
        "A",
        "B"
    ];

    glyphs.iter()
        .collect_into_glyph_order()
        .unwrap()
}

fn main() {
    let (in_path, out_path) = {
        let mut a = env::args().skip(1).take(2);

        match (a.next(), a.next()) {
            (Some(inp), Some(out)) => (inp, out),
            _ => {
                eprintln!("usage: compile <input> <output>");
                ::std::process::exit(1);
            }
        }
    };

    let f = File::open(&in_path).unwrap();

    let statements = parser::parse_file(f).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    let glyph_order = fealib_builder_glyph_order();
    compiler::compile(glyph_order, &statements, &mut buf);

    let mut f = File::create(&out_path).unwrap();
    f.write(&buf).unwrap();
}
