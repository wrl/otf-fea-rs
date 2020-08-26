pub trait Pred: Copy {
    fn pred(self) -> Self;
}

impl Pred for u16 {
    fn pred(self) -> Self {
        self - 1
    }
}

pub struct ContiguousRanges<T, I>
    where T: Pred + Eq,
          I: Iterator<Item = T>
{
    inner: I,
    start: Option<T>
}

impl<T, I> Iterator for ContiguousRanges<T, I>
    where T: Pred + Eq + Copy,
          I: Iterator<Item = T>
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let start = match self.start.take() {
            Some(x) => x,

            None => match self.inner.next() {
                Some(x) => x,
                None => return None
            }
        };

        let mut prev = start;

        for x in &mut self.inner {
            if prev != x.pred() {
                self.start = Some(x);
                return Some((start, prev));
            }

            prev = x;
        }

        Some((start, prev))
    }
}

pub trait ContiguousRangesIterExt<T>: Iterator<Item = T> + Sized
    where T: Pred + Eq
{
    fn contiguous_ranges(self) -> ContiguousRanges<T, Self> {
        ContiguousRanges {
            inner: self,
            start: None
        }
    }
}

impl<T, I> ContiguousRangesIterExt<T> for I
    where T: Pred + Eq,
          I: Iterator<Item = T> + Sized
{ }
