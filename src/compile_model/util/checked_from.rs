use std::any::type_name;
use std::convert::TryInto;

use crate::compile_model::error::*;


pub trait CheckedFrom<T, E>: Sized {
    fn checked_from(scope: impl Into<String>, item: &'static str, t: T) -> Result<Self, E>;
}

impl<F> CheckedFrom<F, EncodeError> for u16
    where F: TryInto<Self> + Into<usize> + Copy
{
    fn checked_from(scope: impl Into<String>, item: &'static str, value: F) -> Result<Self, EncodeError> {
        value.try_into()
            .map_err(|_| EncodeError::U16Overflow {
                scope: scope.into(),
                item,
                value: value.into()
            })
    }
}

impl<T, F> CheckedFrom<F, CompileError> for T
    where F: TryInto<Self> + TryInto<isize> + Copy
{
    fn checked_from(scope: impl Into<String>, item: &'static str, value: F) -> Result<Self, CompileError> {
        value.try_into()
            .map_err(|_| CompileError::Overflow {
                ty: type_name::<T>(),
                scope: scope.into(),
                item,
                value: value.try_into().unwrap_or(0)
            })
    }
}

pub trait CheckedInto<T, E>: Sized {
    fn checked_into(self, scope: impl Into<String>, item: &'static str) -> Result<T, E>;
}

impl<T, F, E> CheckedInto<T, E> for F
    where T: CheckedFrom<F, E>
{
    #[inline]
    fn checked_into(self, scope: impl Into<String>, item: &'static str) -> Result<T, E> {
        T::checked_from(scope, item, self)
    }
}
