pub(crate) enum Either2<A,B> {
    A(A),
    B(B)
}

impl<A, B, T> Iterator for Either2<A, B>
    where A: Iterator<Item = T>,
          B: Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self {
            Either2::A(inner) => inner.next(),
            Either2::B(inner) => inner.next()
        }
    }
}

pub(crate) enum Either3<A,B,C> {
    A(A),
    B(B),
    C(C)
}

impl<A, B, C, T> Iterator for Either3<A, B, C>
    where A: Iterator<Item = T>,
          B: Iterator<Item = T>,
          C: Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self {
            Either3::A(inner) => inner.next(),
            Either3::B(inner) => inner.next(),
            Either3::C(inner) => inner.next()
        }
    }
}
