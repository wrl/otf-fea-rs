use ascii::AsciiChar;


pub(crate) type TagStorage = [AsciiChar; 4];

fn tag_storage_from_bytes(v: &[u8]) -> Result<TagStorage, ::ascii::ToAsciiCharError> {
    let mut tag = [::ascii::AsciiChar::Space; 4];

    let iter = v.iter()
        .map(|x| ::ascii::AsciiChar::from_ascii(*x))
        .take(4);

    for (i, c) in iter.enumerate() {
        tag[i] = c?;
    }

    Ok(tag)
}

fn tag_storage_from_u32(v: u32) -> Result<TagStorage, ::ascii::ToAsciiCharError> {
    let mut tag = [::ascii::AsciiChar::Space; 4];

    tag[0] = ::ascii::AsciiChar::from_ascii((v >> 24) & 0xFF)?;
    tag[1] = ::ascii::AsciiChar::from_ascii((v >> 16) & 0xFF)?;
    tag[2] = ::ascii::AsciiChar::from_ascii((v >>  8) & 0xFF)?;
    tag[3] = ::ascii::AsciiChar::from_ascii((v >>  0) & 0xFF)?;

    Ok(tag)
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

// from https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963, thanks!
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! tag_type {
    ($type:ident, $mac:ident) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
        pub struct $type(pub TagStorage);

        impl $type {
            pub fn from_bytes(v: &[u8]) -> Result<Self, ::ascii::ToAsciiCharError> {
                tag_storage_from_bytes(v)
                    .map($type)
            }
        }

        impl ::std::convert::TryFrom<u32> for $type {
            type Error = ::ascii::ToAsciiCharError;

            fn try_from(v: u32) -> Result<Self, Self::Error> {
                tag_storage_from_u32(v)
                    .map($type)
            }
        }

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

        with_dollar_sign! {
            ($d:tt) => {
                #[macro_export]
                macro_rules! $mac {
                    ($d($d args:tt),+) => {
                        $crate::$type($crate::tag_storage!($d($d args),+))
                    }
                }
            }
        }
    }
}

tag_type!(Tag, tag);
tag_type!(FeatureTag, feature_tag);
tag_type!(ScriptTag, script_tag);
tag_type!(LanguageTag, language_tag);

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
