use crate::Tag;

#[derive(Debug)]
pub struct TTFTagged<T>(Tag, T);

impl<T> TTFTagged<T> {
    #[inline]
    pub fn new(tag: Tag, inner: T) -> Self {
        Self(tag, inner)
    }
}
