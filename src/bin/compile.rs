use std::io::prelude::*;
use std::borrow::Cow;
use std::fs::File;
use std::env;

use otf_fea_rs::{
    GlyphOrder,
    IntoGlyphOrder,
    parser,
    compiler
};

fn fealib_builder_glyph_order() -> GlyphOrder {
    let glyphs = "
        .notdef space slash fraction semicolon period comma ampersand
        quotedblleft quotedblright quoteleft quoteright
        zero one two three four five six seven eight nine
        zero.oldstyle one.oldstyle two.oldstyle three.oldstyle
        four.oldstyle five.oldstyle six.oldstyle seven.oldstyle
        eight.oldstyle nine.oldstyle onequarter onehalf threequarters
        onesuperior twosuperior threesuperior ordfeminine ordmasculine
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
        a b c d e f g h i j k l m n o p q r s t u v w x y z
        A.sc B.sc C.sc D.sc E.sc F.sc G.sc H.sc I.sc J.sc K.sc L.sc M.sc
        N.sc O.sc P.sc Q.sc R.sc S.sc T.sc U.sc V.sc W.sc X.sc Y.sc Z.sc
        A.alt1 A.alt2 A.alt3 B.alt1 B.alt2 B.alt3 C.alt1 C.alt2 C.alt3
        a.alt1 a.alt2 a.alt3 a.end b.alt c.mid d.alt d.mid
        e.begin e.mid e.end m.begin n.end s.end z.end
        Eng Eng.alt1 Eng.alt2 Eng.alt3
        A.swash B.swash C.swash D.swash E.swash F.swash G.swash H.swash
        I.swash J.swash K.swash L.swash M.swash N.swash O.swash P.swash
        Q.swash R.swash S.swash T.swash U.swash V.swash W.swash X.swash
        Y.swash Z.swash
        f_l c_h c_k c_s c_t f_f f_f_i f_f_l f_i o_f_f_i s_t f_i.begin
        a_n_d T_h T_h.swash germandbls ydieresis yacute breve
        grave acute dieresis macron circumflex cedilla umlaut ogonek caron
        damma hamza sukun kasratan lam_meem_jeem noon.final noon.initial
        by feature lookup sub table uni0327 uni0328 e.fina
    ";

    let cids = (800..1001usize)
        .map(|cid| Cow::Owned(format!("cid{:05}", cid)));

    glyphs
        .split_whitespace().map(Cow::Borrowed)
        .chain(cids)
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
