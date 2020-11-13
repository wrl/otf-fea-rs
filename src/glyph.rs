use std::fmt;

use thiserror::Error;

use ascii::{
    AsciiChar,
    AsciiStr,
    AsAsciiStr,
    AsAsciiStrError
};

use arrayvec::ArrayVec;


#[derive(Debug, Error)]
pub enum GlyphError {
    #[error("glyph name {0} starts with an invalid character")]
    InvalidStartingCharacter(GlyphName),

    #[error("glyph name can be a maximum of 63 characters long")]
    GlyphNameTooLong,

    #[error(transparent)]
    GlyphNameNotAscii(#[from] AsAsciiStrError)
}

pub(crate) type GlyphNameStorage = ArrayVec::<[AsciiChar; 63]>;

pub(crate) trait GlyphNameStorageToStr {
    fn as_str(&self) -> &str;
}

impl GlyphNameStorageToStr for GlyphNameStorage {
    fn as_str(&self) -> &str {
        self.as_ascii_str().unwrap().as_str()
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphName(pub GlyphNameStorage);

impl fmt::Debug for GlyphName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlyphName(\"")?;

        for ch in &self.0 {
            write!(f, "{}", ch)?;
        }

        write!(f, "\")")
    }
}

impl fmt::Display for GlyphName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in &self.0 {
            write!(f, "{}", ch)?;
        }

        Ok(())
    }
}

#[inline]
pub(crate) fn glyph_character_valid(c: u8, first_character: bool, development_names: bool) -> bool
{
    match c {
        (b'a' ..= b'z') | (b'A' ..= b'Z') | b'_' => true,
        b'.' | (b'0' ..= b'9') if !first_character => true,

        b'*' | b'+' | b'-' | b':' | b'^' | b'|' | b'~'
            if development_names => true,

        _ => false
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphCID(pub usize);

impl fmt::Debug for GlyphCID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlyphCID({})", self.0)
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum GlyphRef {
    Name(GlyphName),
    CID(GlyphCID)
}

impl GlyphRef {
    pub fn from_cid(cid: usize) -> Self {
        Self::CID(GlyphCID(cid))
    }

    pub fn from_name(name: &str) -> Result<Self, GlyphError> {
        let astr = AsciiStr::from_ascii(name)?;

        let mut n = GlyphNameStorage::new();

        n.try_extend_from_slice(astr.as_slice())
            .map_err(|_| GlyphError::GlyphNameTooLong)?;

        // FIXME: development names??
        if !glyph_character_valid(astr[0].as_byte(), true, true) && astr != ".notdef" {
            return Err(GlyphError::InvalidStartingCharacter(GlyphName(n)));
        }

        Ok(Self::Name(GlyphName(n)))
    }
}

impl fmt::Debug for GlyphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlyphRef::Name(ref name) => {
                write!(f, "GlyphRef(name = \"")?;

                for ch in &name.0 {
                    write!(f, "{}", ch)?;
                }

                write!(f, "\")")
            },

            GlyphRef::CID(ref cid) =>
                write!(f, "GlyphRef(CID = {})", cid.0)
        }
    }
}
