use core::marker::PhantomData;

use typenum::consts::True;
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

#[derive(Clone, Copy, Debug)]
pub struct Bounded<N, L, U> {
    pub val: N,
    pub _lower: PhantomData<L>,
    pub _upper: PhantomData<U>,
}

impl<N, L, U> Bounded<N, L, U>
where
    N: Clone + Copy + PartialOrd,
    L: ReifyTo<N>,
    U: ReifyTo<N>,
    U: IsGreater<L, Output = True>,
{
    pub fn new(val: N) -> Option<Self> {
        if val >= L::reify() && val <= U::reify() {
            Some(Bounded {
                val: val,
                _lower: PhantomData,
                _upper: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn checked<V: Unsigned>() -> Self
    where
        V: IsLessOrEqual<U, Output = True>,
        V: IsGreaterOrEqual<L, Output = True>,
        V: ReifyTo<N>,
    {
        Self {
            val: V::reify(),
            _lower: PhantomData,
            _upper: PhantomData,
        }
    }
}

pub trait ReifyTo<T> {
    fn reify() -> T;
}

impl<T: Unsigned> ReifyTo<u8> for T {
    fn reify() -> u8 {
        T::U8
    }
}

impl<T: Unsigned> ReifyTo<u16> for T {
    fn reify() -> u16 {
        T::U16
    }
}

impl<T: Unsigned> ReifyTo<u32> for T {
    fn reify() -> u32 {
        T::U32
    }
}

impl<T: Unsigned> ReifyTo<usize> for T {
    fn reify() -> usize {
        T::USIZE
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use typenum::consts::{U0, U2};

    #[test]
    fn within_range() {
        let b: Bounded<u8, U0, U2> = Bounded::new(1).unwrap();
        assert_eq!(b.val, 1);
    }

    #[test]
    fn contravenes() {
        let b: Option<Bounded<u8, U0, U2>> = Bounded::new(5);
        assert!(b.is_none());
    }
}
