use std::fmt;
use std::cmp;

use ascii::{
    AsciiChar,
    ToAsciiCharError
};


#[derive(Eq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct Tag(pub [AsciiChar; 4]);

impl Tag {
    pub fn from_bytes(v: &[u8]) -> Result<Self, ToAsciiCharError> {
        let mut tag = Tag([AsciiChar::Space; 4]);

        let iter = v.iter()
            .map(|x| AsciiChar::from_ascii(*x))
            .take(4);

        for (i, c) in iter.enumerate() {
            tag.0[i] = c?;
        }

        Ok(tag)
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag(\"{}{}{}{}\")",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl cmp::PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[macro_export]
macro_rules! tag {
    ($a:ident, $b:ident, $c:ident, $d:ident) => {
        $crate::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::$d
        ])
    };

    ($a:ident, $b:ident, $c:ident) => {
        $crate::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::Space
        ])
    };

    ($a:ident, $b:ident) => {
        $crate::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ])
    };

    ($a:ident) => {
        $crate::Tag([
            ascii::AsciiChar::$a,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ])
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_equality() {
        assert_eq!(tag!(a), tag!(a));
        assert_eq!(tag!(a,b), tag!(a,b));
        assert_eq!(tag!(a,b,c), tag!(a,b,c));
        assert_eq!(tag!(a,b,c,d), tag!(a,b,c,d));

        assert_ne!(tag!(a), tag!(w));
        assert_ne!(tag!(a,b), tag!(w,x));
        assert_ne!(tag!(a,b,c), tag!(w,x,y));
        assert_ne!(tag!(a,b,c,d), tag!(w,x,y,z));
    }
}
