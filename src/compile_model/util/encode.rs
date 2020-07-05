use crate::compile_model::{
    TTFEncode,
    EncodeBuf
};

pub(crate) struct WriteIntoPool<'a, I> {
    iter: I,
    buf: &'a mut EncodeBuf
}

impl<'a, I, T> Iterator for WriteIntoPool<'a, I>
    where I: Iterator<Item = &'a T>,
          T: TTFEncode + 'a
{
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            self.buf.append(item).unwrap() as u16
        })
    }
}

pub(crate) trait TTFEncodeExt<'a, T>: Iterator<Item = &'a T>
    where T: TTFEncode + 'a,
          Self: Sized + 'a
{
    #[inline]
    fn write_into_ttf_pool(self, buf: &mut EncodeBuf) -> WriteIntoPool<Self> {
        WriteIntoPool { iter: self, buf }
    }
}

impl<'a, T, I> TTFEncodeExt<'a, T> for I
    where T: TTFEncode + 'a,
          I: Iterator<Item = &'a T> + Sized + 'a
{ }
