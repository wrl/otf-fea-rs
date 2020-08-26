use ascii::{
    AsciiChar,
    ToAsciiCharError
};


pub(crate) type TagStorage = [AsciiChar; 4];

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Tag(pub TagStorage);

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

#[macro_export]
macro_rules! tag_storage {
    ($a:ident, $b:ident, $c:ident, $d:ident) => {
        [
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::$d
        ]
    };

    ($a:ident, $b:ident, $c:ident) => {
        [
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::$c,
            ascii::AsciiChar::Space
        ]
    };

    ($a:ident, $b:ident) => {
        [
            ascii::AsciiChar::$a,
            ascii::AsciiChar::$b,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ]
    };

    ($a:ident) => {
        [
            ascii::AsciiChar::$a,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space,
            ascii::AsciiChar::Space
        ]
    };
}

#[macro_export]
macro_rules! tag_impl {
    ($type:ty) => {
        impl ::std::fmt::Debug for $type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, concat!(stringify!($type), "(\"{}{}{}{}\")"),
                self.0[0], self.0[1], self.0[2], self.0[3])
            }
        }

        impl ::std::fmt::Display for $type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}{}{}{}",
                    self.0[0], self.0[1], self.0[2], self.0[3])
            }
        }
    }
}

tag_impl!(Tag);

#[macro_export]
macro_rules! tag {
    ($($args:tt),+) => {
        $crate::Tag($crate::tag_storage!($($args),+))
    }
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
